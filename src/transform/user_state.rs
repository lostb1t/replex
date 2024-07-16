use crate::{
    config::Config,
    models::*,
    plex_client::{PlexClient},
};
use super::Transform;
use async_trait::async_trait;

#[derive(Default, Debug)]
pub struct UserStateTransform;

#[async_trait]
impl Transform for UserStateTransform {
    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexContext,
    ) {
        let config: Config = Config::figment().extract().unwrap();
        if !config.disable_user_state && !config.disable_leaf_count {
            return;
        }
        if item.is_hub() {
            for child in item.children_mut() {
                if config.disable_user_state {
                    child.user_state = Some(SpecialBool::new(false));
                }
                if config.disable_leaf_count {
                    child.leaf_count = None;
                }
            }
        } else {
            if config.disable_user_state {
                item.user_state = Some(SpecialBool::new(false));
            }
            if config.disable_leaf_count {
                item.leaf_count = None;
            }
        }
    }
}
