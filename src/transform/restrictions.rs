use async_trait::async_trait;
use crate::{
    config::Config,
    models::*,
    plex_client::{PlexClient},
};

use super::Transform;
#[derive(Default)]
pub struct HubRestrictionTransform;

#[async_trait]
impl Transform for HubRestrictionTransform {
    async fn filter_metadata(
        &self,
        item: MetaData,
        plex_client: PlexClient,
        options: PlexContext,
    ) -> bool {
        let config: Config = Config::figment().extract().unwrap();
        
        if !config.hub_restrictions {
            return true;
        }

        if item.is_hub() && !item.is_collection_hub() {
            return true;
        }
        
        if !item.is_hub() {
            return true;
        }
        
        if item.size.unwrap() == 0 {
            return false;
        }

        let section_id: i64 = item.library_section_id.unwrap_or_else(|| {
            item.hub_identifier.clone().unwrap().split('.').collect::<Vec<&str>>()[2].parse().unwrap()
        });

        //let start = Instant::now();
        let mut custom_collections = plex_client
            .clone()
            .get_cached(
                plex_client.get_section_collections(section_id),
                format!("sectioncollections:{}", section_id).to_string(),
            )
            .await
            .unwrap();

        //println!("Elapsed time: {:.2?}", start.elapsed());
        let custom_collections_ids: Vec<String> = custom_collections
            .media_container
            .children()
            .iter()
            .map(|c| c.rating_key.clone().unwrap())
            .collect();

        custom_collections_ids.contains(
            &item
                .hub_identifier
                .clone()
                .unwrap()
                .split('.')
                .last()
                .unwrap()
                .to_owned(),
        )
    }
}