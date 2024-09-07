use crate::{
    config::Config,
    models::*,
    plex_client::{PlexClient},
};
use super::Transform;
use async_trait::async_trait;
use itertools::Itertools;

#[derive(Default, Debug, Clone)]
pub struct LibraryInterleaveTransform {
    pub collection_ids: Vec<u32>,
    pub offset: i32,
    pub limit: i32,
}

#[async_trait]
impl Transform for LibraryInterleaveTransform {
    async fn transform_mediacontainer(
        &self,
        mut item: MediaContainer,
        plex_client: PlexClient,
        options: PlexContext,
    ) -> MediaContainer {
        let config: Config = Config::figment().extract().unwrap();
        if !config.interleave {
            return item;
        }
        let mut children: Vec<MetaData> = vec![];
        let mut total_size = 0;

        for id in self.collection_ids.clone() {
            let collection = plex_client
                .clone()
                .get_cached(
                    plex_client.get_collection(id as i32),
                    format!("collection:{}", id.to_string()),
                )
                .await
                .unwrap();
        
            //match c {
            //    Ok(v) =>,
            //    Err(err) =>
            //}
        
            let mut c = plex_client
                .clone()
                .get_cached(
                    plex_client.get_collection_children(
                        id as i64,
                        Some(self.offset),
                        Some(self.limit),
                    ),
                    format!(
                        "get_collection_children:{}:{}:{}",
                        id, self.offset, self.limit
                    ),
                )
                .await
                .unwrap();
            
            // should have proper errors but lets assume not found so no access
            //match c {
            //    Ok(v) =>,
            //    Err(err) =>
            //}


            if collection.media_container.exclude_watched() {
                c.media_container.children_mut().retain(|x| !x.is_watched());
            }

            total_size += c.media_container.children().len() as i32;

            match children.is_empty() {
                false => {
                    children = children
                        .into_iter()
                        .interleave(c.media_container.children())
                        .collect::<Vec<MetaData>>();
                }
                true => children.append(&mut c.media_container.children()),
            }
        }
        item.total_size = Some(total_size);
        // always metadata
        item.metadata = children;
        item
    }
}