use crate::{
    config::Config,
    models::*,
    plex_client::{self, PlexClient},
    utils::*,
};

use super::Transform;
use async_trait::async_trait;
use itertools::Itertools;

#[derive(Default, Debug)]
pub struct HubInterleaveTransform;

#[async_trait]
impl Transform for HubInterleaveTransform {
    async fn transform_mediacontainer(
        &self,
        mut item: MediaContainer,
        plex_client: PlexClient,
        options: PlexContext,
    ) -> MediaContainer {
        let config: Config = Config::figment().extract().unwrap();
        let mut new_hubs: Vec<MetaData> = vec![];
        
        if !config.interleave {
            return item;
        }

        for mut hub in item.children_mut() {
            if hub.size.unwrap() == 0 {
                continue;
            }

            // we only process collection hubs
            if !hub.is_collection_hub() {
                new_hubs.push(hub.to_owned());
                continue;
            }

            //hub.context = Some("hub.home.watchlist_available".to_string());
            //hub.r#type = "clip".to_string();
            // hub.placeholder = Some(SpecialBool::new(true));
            //hub.placeholder = Some(true);

            let p = new_hubs.iter().position(|v| v.title == hub.title);
            // if hub.r#type != "clip" {
            //     hub.r#type = "mixed".to_string();
            // }
            match p {
                Some(v) => {
                    new_hubs[v].key = Some(merge_children_keys(
                        new_hubs[v].key.clone().unwrap(),
                        hub.key.clone().unwrap(),
                    ));
                    let c = new_hubs[v].children();
                    new_hubs[v].set_children(
                        c.into_iter()
                            .interleave(hub.children())
                            .collect::<Vec<MetaData>>(),
                    );
                }
                None => new_hubs.push(hub.to_owned()),
            }
        }
        item.set_children_mut(&mut new_hubs);
        item
    }
}