use crate::{
    config::Config,
    models::*,
    plex_client::{self, PlexClient},
    utils::*,
};

use super::Transform;
use super::MediaStyleTransform;
use super::ClientHeroStyle;
use super::hero_meta;
use async_trait::async_trait;
use futures_util::{
    future::{self, join_all, LocalBoxFuture},
    stream::{FuturesOrdered, FuturesUnordered},
    StreamExt,
};

/// Collections can be called from hubs as a refresh. But also standalone.
/// We need to know if if its hub called and if the hub is hero styled for media.
#[derive(Default, Debug)]
pub struct CollectionStyleTransform {
    pub collection_ids: Vec<u32>,
    pub hub: bool, // if collections is meant for hubs
}

#[async_trait]
impl Transform for CollectionStyleTransform {
    async fn transform_mediacontainer(
        &self,
        mut item: MediaContainer,
        plex_client: PlexClient,
        options: PlexContext,
    ) -> MediaContainer {
        let mut collection_details = plex_client
            .clone()
            .get_cached(
                plex_client.get_collection(self.collection_ids[0] as i32),
                format!("collection:{}", self.collection_ids[0].to_string()),
            )
            .await;

        if collection_details.is_ok()
            && collection_details
                .unwrap()
                .media_container
                .children()
                .get(0)
                .unwrap()
                .has_label("REPLEXHERO".to_string())
        {
            // let mut futures = FuturesOrdered::new();
            // let now = Instant::now();

            let mut style = ClientHeroStyle::from_context(options.clone());

            item.meta = Some(hero_meta());

            let mut futures = FuturesOrdered::new();
            for mut child in item.children() {
                if style.child_type.clone().is_some() {
                    child.r#type = style.child_type.clone().unwrap();
                }

                let client = plex_client.clone();
                let _options = options.clone();
                futures.push_back(async move {
                    let mut c = child.clone();
                    let transform = MediaStyleTransform { style: Style::Hero };
                    transform
                        .transform_metadata(&mut c, client, _options)
                        .await;
                    c
                });
            }
            let children: Vec<MetaData> = futures.collect().await;
            item.set_children(children);
        }
        item
    }
}