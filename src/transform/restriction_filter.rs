use async_trait::async_trait;
use crate::{
    config::Config,
    models::*,
    plex_client::{PlexClient},
};

use super::Filter;
#[derive(Default)]
pub struct HubRestrictionFilter;

#[async_trait]
impl Filter for HubRestrictionFilter {
    async fn filter_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexContext,
    ) -> bool {
        let config: Config = Config::figment().extract().unwrap();
        
        if !config.hub_restrictions {
            return false;
        }

        if item.is_hub() && !item.is_collection_hub() {
            return false;
        }
        
        if !item.is_hub() {
            return false;
        }
        
        if !item.size.unwrap() == 0 {
            return true;
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

        !custom_collections_ids.contains(
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