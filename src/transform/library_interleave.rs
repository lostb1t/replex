use crate::{
    config::Config,
    models::*,
    plex_client::{self, PlexClient},
    utils::*,
};
use super::Transform;
use async_trait::async_trait;
use itertools::Itertools;

#[derive(Default, Debug, Clone)]
pub struct LibraryInterleaveTransform {
    pub collection_ids: Vec<u32>,
    pub offset: i32,
    pub limit: i32,
    // pub remove_watched: bool,
}

// #[async_recursion]
// pub async fn load_collection_children(
//     id: u32,
//     offset: i32,
//     limit: i32,
//     original_limit: i32,
//     client: PlexClient,
// ) -> anyhow::Result<MediaContainerWrapper<MediaContainer>> {
//     let config: Config = Config::figment().extract().unwrap();
//     let mut c = client
//         .get_collection_children(id, Some(offset), Some(limit))
//         .await?;

//     if !config.include_watched {
//         c.media_container.children_mut().retain(|x| !x.is_watched());

//         let children_lenght = c.media_container.children_mut().len() as i32;
//         // dbg!(children_lenght);
//         // dbg!(limit);
//         // dbg!(c.media_container.total_size.unwrap());
//         // dbg!(offset);
//         // dbg!(limit);
//         // dbg!(children_lenght);
//         // dbg!(c.media_container.total_size);
//         // dbg!("-------");
//         let total_size = c.media_container.total_size.unwrap();
//         if children_lenght < original_limit
//             && total_size > offset + limit && offset < total_size
//         {
//             //dbg!("recursive");
//             // self.clone().load_collection_children();
//             // load more
//             return load_collection_children(
//                 id,
//                 offset,
//                 limit + 10,
//                 original_limit,
//                 client.clone(),
//             ).await;
//         }
//     }
//     Ok(c)
// }

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

            let collection = plex_client
                .clone()
                .get_cached(
                    plex_client.get_collection(id as i32),
                    format!("collection:{}", id.to_string()),
                )
                .await
                .unwrap();

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