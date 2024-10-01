use crate::{
    config::Config,
    models::*,
    plex_client::{PlexClient},
};
use super::Transform;
use async_trait::async_trait;

#[derive(Default, Debug, Clone)]
pub struct HubReorderTransform {
    pub collection_ids: Vec<u32>,
}

#[async_trait]
impl Transform for HubReorderTransform {
    async fn transform_mediacontainer(
        &self,
        mut item: MediaContainer,
        plex_client: PlexClient,
        options: PlexContext,
    ) -> MediaContainer {
        let config: Config = Config::figment().extract().unwrap();
        if !config.custom_sorting.unwrap() { // Assuming this flag is in config.rs
            return item;
        }

        let mut children: Vec<MetaData> = item.metadata.clone();

        // Create a hash map for the custom order and any hubs not specified
        let mut ordered_children = vec![];
        let mut unordered_children = vec![];

        for child in children.drain(..) {
            if let Some(hub_index) = config.custom_sorting.unwrap().position(|&id| id == child.rating_key) {
                ordered_children.push((hub_index, child));
            } else {
                unordered_children.push(child);
            }
        }

        // Sort the ordered children by their position in the hub_order
        ordered_children.sort_by_key(|(hub_index, _)| *hub_index);

        // Flatten the sorted children and append the unordered ones at the end
        let sorted_children: Vec<MetaData> = ordered_children
            .into_iter()
            .map(|(_, child)| child)
            .chain(unordered_children.into_iter())
            .collect();

        item.metadata = sorted_children;
        item
    }
}