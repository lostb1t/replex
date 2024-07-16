use crate::{
    models::*,
    plex_client::{PlexClient},
};
use super::Transform;
use async_trait::async_trait;

#[derive(Default, Debug)]
pub struct HubSectionDirectoryTransform;

/// Some sections return a directory instead of video. We dont want that
#[async_trait]
impl Transform for HubSectionDirectoryTransform {
    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexContext,
    ) {
        if item.is_collection_hub() && !item.directory.is_empty() {
            let childs = item.children();
            item.directory = vec![];
            item.video = childs;
        }

        // if item.is_collection_hub() {
        //     let childs = item.children();
        //     item.metadata = vec![];
        //     item.video = childs;
        // }
    }
}