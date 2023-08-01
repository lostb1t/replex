use async_trait::async_trait;
use futures_util::{
    future::{self, join_all, LocalBoxFuture},
    stream::FuturesUnordered,
};
use std::sync::Arc;
// use crate::models::*;
use crate::{models::*, plex_client::PlexClient, utils::*};
use typed_builder::TypedBuilder;

// pub struct MetaDataTest {
//     pub id: i32
// }

#[async_trait]
pub trait Transform: Send + Sync + 'static {
    async fn transform(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    );
}

#[async_trait]
pub trait Filter: Send + Sync + 'static {
    async fn filter(
        &self,
        item: MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    ) -> bool;
}

// #[derive(TypedBuilder)]
// #[derive(Default)]
#[derive(Clone)]
pub struct TransformBuilder {
    pub plex_client: PlexClient,
    pub options: PlexParams,
    pub mix: bool,
    pub transforms: Vec<Arc<dyn Transform>>,
    pub filters: Vec<Arc<dyn Filter>>,
}

impl TransformBuilder {
    #[inline]
    pub fn new(plex_client: PlexClient, options: PlexParams) -> Self {
        Self {
            transforms: Vec::new(),
            filters: Vec::new(),
            mix: false,
            plex_client,
            options,
        }
    }

    #[inline]
    pub fn with_transform<T: Transform>(mut self, transform: T) -> Self {
        self.transforms.push(Arc::new(transform));
        self
    }

    #[inline]
    pub fn with_filter<T: Filter>(mut self, filter: T) -> Self {
        self.filters.push(Arc::new(filter));
        self
    }

    // pub fn merge(mut self, container_left, container_right) -> Self {
    //     self.mix = true;
    //     self
    // }

    // TODO: join async filters
    pub async fn apply_to(
        self,
        container: &mut MediaContainerWrapper<MediaContainer>,
    ) {
        let mut filtered_childs: Vec<MetaData> = vec![];
        'outer: for item in container.media_container.test() {
            for filter in self.filters.clone() {
                if !filter
                    .filter(
                        item.to_owned(),
                        self.plex_client.clone(),
                        self.options.clone(),
                    )
                    .await
                {
                    break 'outer;
                }
            }
            filtered_childs.push(item.to_owned());
        }
        container.media_container.set_children(filtered_childs);

        // if self.mix {
        //     for id in reversed {
        //         let mut c = plex_client
        //             .get_collection_children(id, offset.clone(), limit.clone())
        //             .await
        //             .unwrap();
        //         total_size += c.media_container.total_size.unwrap();
        //         match children.is_empty() {
        //             false => {
        //                 children = children
        //                     .into_iter()
        //                     .interleave(c.media_container.children())
        //                     .collect::<Vec<MetaData>>();
        //             }
        //             true => children.append(&mut c.media_container.children()),
        //         }
        //     }
        // }

        // for t in self.filters.clone() {
        // for item in container.media_container.test() {
        //     let futures = for filter in self.filters.clone().iter().map(
        //         |x: Filter| {
        //             x.filter(
        //                 item,
        //                 self.plex_client.clone(),
        //                 self.options.clone(),
        //             )
        //         }
        //     );
        //     let results = future::join_all(futures).await;
        // }

        for t in self.transforms.clone() {
            let futures = container.media_container.test().iter_mut().map(
                |x: &mut MetaData| {
                    t.transform(
                        x,
                        self.plex_client.clone(),
                        self.options.clone(),
                    )
                },
            );
            future::join_all(futures).await;
        }

        if container.media_container.size.is_some() {
            container.media_container.size = Some(
                container.media_container.test().len().try_into().unwrap(),
            );
        }
    }
}

// const T: usize;
#[derive(Default)]
pub struct CollectionPermissionFilter;

#[async_trait]
impl Filter for CollectionPermissionFilter {
    async fn filter(
        &self,
        item: MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    ) -> bool {
        tracing::debug!("filter collection permissions");
        if item.is_hub() && !item.is_collection_hub() {
            return true;
        }
        // dbg!(&metadata);
        let section_id: u32 = item.library_section_id.unwrap_or_else(|| {
            item.clone()
                .test()
                .get(0)
                .unwrap()
                .library_section_id
                .expect("Missing Library section id")
        });

        // let mut custom_collections = plex_client.get_section_collections(section_id).await.unwrap();
        let mut custom_collections = plex_client.clone().get_cached(
            plex_client.get_section_collections(section_id),
            format!("sectioncollections:{}", section_id).to_string(),
        ).await.unwrap();
        let custom_collections_keys: Vec<String> =
            custom_collections.media_container.test().iter().map(|c| c.key.clone()).collect();
        custom_collections_keys.contains(&item.key)
    }
}

#[derive(Default, Debug)]
pub struct StyleTransform;

#[async_trait]
impl Transform for StyleTransform {
    async fn transform(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
        if item.is_collection_hub() {
            let mut collection_details = plex_client
                .clone()
                .get_cached(
                    plex_client.get_collection(get_collection_id_from_child_path(
                        item.key.clone(),
                    )),
                    format!("collection:{}", item.key.clone()).to_string(),
                )
                .await
                .unwrap();

            if collection_details
                .media_container
                .children()
                .get(0)
                .unwrap()
                .has_label("REPLEXHERO".to_string())
            {
                item.style = Some("hero".to_string());

                // for android, as it doesnt listen to hero style on home..... so we make it a clip
                if let Some(platform) = &options.platform {
                    if platform.to_lowercase() == "android" {
                        // dbg!("We got android");
                        // self.meta = Some(Meta {
                        //     r#type: None,
                        //     display_fields: vec![
                        //         DisplayField {
                        //             r#type: Some("movie".to_string()),
                        //             fields: vec!["title".to_string(), "year".to_string()],
                        //         },
                        //         DisplayField {
                        //             r#type: Some("show".to_string()),
                        //             fields: vec!["title".to_string(), "year".to_string()],
                        //         },
                        //     ],
                        // });
                        item.r#type = "clip".to_string();
                    }
                }
            }
        }
    }
}

// #[derive(Default, Debug)]
// pub struct MixHubTransform;

// #[async_trait]
// impl Transform for MixHubTransform {
//     async fn transform(
//         &self,
//         item: &mut MetaData,
//         plex_client: PlexClient,
//         options: PlexParams,
//     ) {
//         for mut hub in item {
//             let p = new_collections.iter().position(|v| v.title == hub.title);

//             if hub.r#type != "clip" {
//                 hub.r#type = "mixed".to_string();
//             }

//             match p {
//                 Some(v) => {
//                     new_collections[v].key =
//                         merge_children_keys(new_collections[v].key.clone(), hub.key.clone());
//                     let c = new_collections[v].children();
//                     // let h = hub.metadata;
//                     new_collections[v].set_children(
//                         c.into_iter()
//                             .interleave(hub.children())
//                             .collect::<Vec<MetaData>>(),
//                     );
//                 }
//                 None => new_collections.push(hub),
//             }
//         }
//     }
// }

// example usage

// metadata = MetaData {
//     id: 34
// }
// transform = TransformBuilder::builder().transforms(CollectionPermissionsTransform::new());
