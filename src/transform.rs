use crate::{config::Config, models::*, plex_client::PlexClient, utils::*};
use async_recursion::async_recursion;
use async_trait::async_trait;
use futures_util::{
    future::{self, join_all, LocalBoxFuture},
    stream::{FuturesUnordered, FuturesOrdered}, StreamExt,
};
use itertools::Itertools;
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio::time::Instant;

#[async_trait]
pub trait Transform: Send + Sync + 'static {
    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
    }
    async fn transform_mediacontainer(
        &self,
        item: &mut MediaContainer,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
    }
}

#[async_trait]
pub trait Filter: Send + Sync + 'static {
    async fn filter_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    ) -> bool {
        true
    }
    async fn filter_mediacontainer(
        &self,
        item: MediaContainer,
        plex_client: PlexClient,
        options: PlexParams,
    ) -> bool {
        true
    }
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
        // self.transforms.insert(0, Arc::new(transform));
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

    #[async_recursion]
    pub async fn apply_to_metadata(
        &self,
        metadata: &mut Vec<MetaData>,
    ) -> Vec<MetaData> {
        let mut filtered_childs: Vec<MetaData> = vec![];
        'outer: for mut item in metadata {
            for filter in self.filters.clone() {
                // dbg!("filtering");
                if !filter
                    .filter_metadata(
                        item,
                        self.plex_client.clone(),
                        self.options.clone(),
                    )
                    .await
                {
                    continue 'outer;
                }
            }

            if !item.children().is_empty() {
                // dbg!(item.test().len());
                let childs = self.apply_to_metadata(item.children_mut()).await;
                // dbg!(childs.len());
                // dbg!(&item.test());
                item.set_children(childs);
                // item.set_children(self.apply_to_metadata(item.test()).await);
            }

            filtered_childs.push(item.to_owned());
        }

        return filtered_childs;
        // dbg!(&metadata.len());
    }

    // TODO: join async filters
    pub async fn apply_to(
        self,
        container: &mut MediaContainerWrapper<MediaContainer>,
    ) {
        // let mut filtered_childs: Vec<MetaData> = vec![];
        // 'outer: for item in container.media_container.test() {
        //     for filter in self.filters.clone() {
        //         if !filter
        //             .filter_metadata(
        //                 item.to_owned(),
        //                 self.plex_client.clone(),
        //                 self.options.clone(),
        //             )
        //             .await
        //         {
        //             break 'outer;
        //         }
        //     }

        //     if !item.test().is_empty() {
        //         for child in item.test() {

        //         }
        //     }

        //     filtered_childs.push(item.to_owned());
        // }

        // let children = self.apply_to_metadata(container.media_container.test());
        // container.media_container.set_children(filtered_childs);

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

        // for t in self.transforms.clone() {
        //     let futures = container.media_container.test().iter_mut().map(
        //         |x: &mut MetaData| {
        //             t.transform_metadata(
        //                 x,
        //                 self.plex_client.clone(),
        //                 self.options.clone(),
        //             )
        //         },
        //     );
        //     future::join_all(futures).await;

        //     t.transform_mediacontainer(
        //         &mut container.media_container,
        //         self.plex_client.clone(),
        //         self.options.clone(),
        //     ).await
        // }
        // for t in self.transforms.clone() {
        //     let futures =
        //         container.media_container.children_mut().iter_mut().map(
        //             |x: &mut MetaData| {
        //                 t.transform_metadata(
        //                     x,
        //                     self.plex_client.clone(),
        //                     self.options.clone(),
        //                 )
        //             },
        //         );
        //     future::join_all(futures).await;

        //     // dont use join as it needs ti be executed in order
        //     t.transform_mediacontainer(
        //         &mut container.media_container,
        //         self.plex_client.clone(),
        //         self.options.clone(),
        //     )
        //     .await
        // }

        for t in self.transforms.clone() {
            // dbg!(&t);
            t.transform_mediacontainer(
                &mut container.media_container,
                self.plex_client.clone(),
                self.options.clone(),
            )
            .await
        }

        for item in container.media_container.children_mut() {
            for t in self.transforms.clone() {
                t.transform_metadata(
                    item,
                    self.plex_client.clone(),
                    self.options.clone(),
                )
                .await;
            }
        }

        // future::join_all(futures).await;

        // dont use join as it needs ti be executed in order

        // }

        // let mut set = tokio::task::JoinSet::new();
        // for t in self.transforms.clone() {
        //     for item in container.media_container.children_mut() {
        //             set.spawn(t.transform_metadata(
        //                 item,
        //                 self.plex_client.clone(),
        //                 self.options.clone(),
        //             )
        //         );
        //     };
        //     // let futures =
        //     //     container.media_container.children_mut().iter_mut().map(
        //     //         |x: &mut MetaData| {
        //     //             t.transform_metadata(
        //     //                 x,
        //     //                 self.plex_client.clone(),
        //     //                 self.options.clone(),
        //     //             )
        //     //         },
        //     //     );

        //     //future::join_all(futures).await;
        //     // future::try_join_all(futures).await;

        //     // dont use join as it needs ti be executed in order
        //     t.transform_mediacontainer(
        //         &mut container.media_container,
        //         self.plex_client.clone(),
        //         self.options.clone(),
        //     )
        //     .await
        // }

        // filter behind transform as transform can load in additional data
        let children = container.media_container.children_mut();
        let new_children = self.apply_to_metadata(children).await;
        container.media_container.set_children(new_children);

        if container.media_container.size.is_some() {
            container.media_container.size = Some(
                container
                    .media_container
                    .children_mut()
                    .len()
                    .try_into()
                    .unwrap(),
            );
        }
    }
}

// const T: usize;
#[derive(Default)]
pub struct CollectionHubPermissionFilter;

#[async_trait]
impl Filter for CollectionHubPermissionFilter {
    async fn filter_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    ) -> bool {
        tracing::debug!("filter collection permissions");

        if item.is_hub() && !item.is_collection_hub() {
            return true;
        }
        if !item.is_hub() {
            return true;
        }
        let section_id: u32 = item.library_section_id.unwrap_or_else(|| {
            item.clone()
                .children()
                .get(0)
                .unwrap()
                .library_section_id
                .expect("Missing Library section id")
        });
        // dbg!(section_id);
        // let mut custom_collections = plex_client.get_section_collections(section_id).await.unwrap();
        let mut custom_collections = plex_client
            .clone()
            .get_cached(
                plex_client.get_section_collections(section_id),
                format!("sectioncollections:{}", section_id).to_string(),
            )
            .await
            .unwrap();
        // dbg!(&custom_collections);
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

#[derive(Default, Debug)]
pub struct HubStyleTransform;

#[async_trait]
impl Transform for HubStyleTransform {
    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
        if item.is_collection_hub() {
            let mut collection_details = plex_client
                .clone()
                .get_cached(
                    plex_client
                        .get_collection(get_collection_id_from_hub(item)),
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

#[derive(Default, Debug)]
pub struct HubMixTransform;

#[async_trait]
impl Transform for HubMixTransform {
    async fn transform_mediacontainer(
        &self,
        item: &mut MediaContainer,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
        let config: Config = Config::figment().extract().unwrap();
        let mut new_hubs: Vec<MetaData> = vec![];
        // let mut library_section_id: Vec<Option<u32>> = vec![]; //librarySectionID
        for mut hub in item.children_mut() {
            if !config.include_watched {
                hub.children_mut().retain(|x| !x.is_watched());
            }

            // we only process collection hubs
            if !hub.is_collection_hub() {
                new_hubs.push(hub.to_owned());
                continue;
            }

            let p = new_hubs.iter().position(|v| v.title == hub.title);
            if hub.r#type != "clip" {
                hub.r#type = "mixed".to_string();
            }
            match p {
                Some(v) => {
                    new_hubs[v].key = merge_children_keys(
                        new_hubs[v].key.clone(),
                        hub.key.clone(),
                    );
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
    }
}

#[derive(Default, Debug)]
pub struct LibraryMixTransform {
    pub collection_ids: Vec<u32>,
    pub offset: Option<i32>,
    pub limit: Option<i32>,
    // pub remove_watched: bool,
}

#[async_trait]
impl Transform for LibraryMixTransform {
    async fn transform_mediacontainer(
        &self,
        item: &mut MediaContainer,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
        let config: Config = Config::figment().extract().unwrap();
        let mut children: Vec<MetaData> = vec![];

        for id in self.collection_ids.clone() {
            let mut c = plex_client
                .get_collection_children(
                    id,
                    self.offset.clone(),
                    self.limit.clone(),
                )
                .await
                .unwrap();
            if !config.include_watched {
                c.media_container.children_mut().retain(|x| !x.is_watched());
            }

            // total_size += c.media_container.total_size.unwrap();
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

        // always metadata library
        item.total_size = Some(children.len() as i32);
        item.metadata = children;
    }
}

#[derive(Default, Debug)]
pub struct HubChildrenLimitTransform {
    pub limit: i32,
}

#[async_trait]
impl Transform for HubChildrenLimitTransform {
    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
        let len = self.limit as usize;
        if item.is_collection_hub() {
            let mut children = item.children();
            children.truncate(len);
            item.set_children(children);
        }
    }
}

#[derive(Default, Debug)]
pub struct TMDBArtTransform;

impl TMDBArtTransform {
    pub async fn transform(&self, item: &mut MetaData) {
        let banner = item.get_tmdb_banner().await;
        if banner.is_some() {
            item.art = banner;
        }
    }
    pub async fn apply_tmdb_banner(&self, item: &mut MetaData) -> MetaData {
        let banner = item.get_tmdb_banner().await;
        if banner.is_some() {
            item.art = banner;
        }
        item.to_owned()
    }
}

#[async_trait]
impl Transform for TMDBArtTransform {
    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
        let config: Config = Config::figment().extract().unwrap();

        // let bla = async move |item| {
        //     let banner = item.get_tmdb_banner().await;
        //     if banner.is_some() {
        //         item.art = banner;
        //     }
        //     item.to_owned()
        // }

        if config.tmdb_api_key.is_some() {
            if item.is_hub() && item.style.clone().unwrap() == "hero" {
                // let mut children: Vec<MetaData> = vec![];
    
                let mut futures = FuturesOrdered::new();
                for child in item.children() {
                    futures.push_back(async move {
                        let mut c = child.clone();
                        let banner = child.get_tmdb_banner().await;
                        if banner.is_some() {
                            c.art = banner;
                        }
                        return c
                    });
                }
                // let now = Instant::now();

                let children: Vec<MetaData>  = futures.collect().await;
                item.set_children(children);

            } else {
                // keep this blocking for now. AS its loaded in the background
                self.transform(item).await;
            }
        }
    }
}

#[derive(Default)]
pub struct WatchedFilter;

#[async_trait]
impl Filter for WatchedFilter {
    async fn filter_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    ) -> bool {
        tracing::debug!("filter watched");
        let config: Config = Config::figment().extract().unwrap();
        if config.include_watched {
            return true;
        }

        if !item.is_hub() {
            return !item.is_watched();
        }
        true
    }
}

#[derive(Default, Debug)]
pub struct HubSectionDirectoryTransform;

/// Some sections return a directory instead of video. We dont want that
#[async_trait]
impl Transform for HubSectionDirectoryTransform {
    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
        if item.is_hub() && !item.directory.is_empty() {
            let childs = item.children();
            item.directory = vec![];
            item.video = childs;
        }
    }
}

#[derive(Default, Debug)]
pub struct HubSectionKeyTransform;

/// We point to replex so we can do some transform on the children calls
#[async_trait]
impl Transform for HubSectionKeyTransform {
    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
       
        if item.is_hub() {
            item.key = format!("/replex{}", item.key);
        }
    }
}

// #[derive(Default)]
// pub struct WatchedFilter;

// #[async_trait]
// impl Filter for WatchedFilter {
//     async fn filter_metadata(
//         &self,
//         item: MetaData,
//         plex_client: PlexClient,
//         options: PlexParams,
//     ) -> bool {
//         tracing::debug!("filter watched");
//         let mut children: Vec<MetaData> = vec![];
//         if item.is_hub() {
//             for mut child in self.media_container.children() {
//                 child.remove_watched();
//                 children.push(child);
//             }
//         } else {
//             children = self
//                 .media_container
//                 .children()
//                 .into_iter()
//                 .filter(|c| !c.is_watched())
//                 .collect::<Vec<MetaData>>();
//         }
//         self.media_container.set_children(children);
//         self
//     }
// }
