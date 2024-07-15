use crate::{
    config::Config,
    models::*,
    plex_client::{self, PlexClient},
    utils::*,
};
use super::Transform;
use async_trait::async_trait;

#[derive(Default, Debug)]
pub struct HubWatchedTransform;

#[async_trait]
impl Transform for HubWatchedTransform {
    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexContext,
    ) {
        let config: Config = Config::figment().extract().unwrap();

        if item.is_hub() {
            let exclude_watched = item
                .exclude_watched(plex_client.clone())
                .await
                .unwrap_or(false);

            if exclude_watched {
                item.children_mut().retain(|x| !x.is_watched());
            }
        }
    }
}
