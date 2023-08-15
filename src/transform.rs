use crate::{
    config::Config,
    models::*,
    plex_client::{self, PlexClient},
    utils::*,
};
use async_recursion::async_recursion;
use async_trait::async_trait;
use futures_util::{
    future::{self, join_all, LocalBoxFuture},
    stream::{FuturesOrdered, FuturesUnordered},
    StreamExt,
};
use itertools::Itertools;
use lazy_static::__Deref;
use std::collections::HashMap;
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
        for t in self.transforms.clone() {
            let futures =
                container.media_container.children_mut().iter_mut().map(
                    |x: &mut MetaData| {
                        t.transform_metadata(
                            x,
                            self.plex_client.clone(),
                            self.options.clone(),
                        )
                    },
                );

            future::join_all(futures).await;

            // dont use join as it needs ti be executed in order
            t.transform_mediacontainer(
                &mut container.media_container,
                self.plex_client.clone(),
                self.options.clone(),
            )
            .await
        }

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

// #[derive(Default, Debug)]
// pub struct HubStyleTransform;

// #[async_trait]
// impl Transform for HubStyleTransform {
//     async fn transform_metadata(
//         &self,
//         item: &mut MetaData,
//         plex_client: PlexClient,
//         options: PlexParams,
//     ) {
//         if item.is_collection_hub() {
//             let mut collection_details = plex_client
//                 .clone()
//                 .get_cached(
//                     plex_client
//                         .get_collection(get_collection_id_from_hub(item)),
//                     format!("collection:{}", item.key.clone()).to_string(),
//                 )
//                 .await;

//             if collection_details.is_ok() && collection_details.unwrap()
//                     .media_container
//                     .children()
//                     .get(0)
//                     .unwrap()
//                     .has_label("REPLEXHERO".to_string()) {
//                 item.style = Some("hero".to_string());
//                 // item.meta = Some(Meta {
//                 //     r#type: None,
//                 //     display_fields: vec![
//                 //         DisplayField {
//                 //             r#type: Some("movie".to_string()),
//                 //             image_type: Some("coverArt".to_string()),
//                 //             fields: vec![],
//                 //         },
//                 //         // DisplayField {
//                 //         //     r#type: Some("show".to_string()),
//                 //         //     fields: vec!["title".to_string(), "year".to_string()],
//                 //         // },
//                 //     ],
//                 // });
//                 // for android, as it doesnt listen to hero style on home..... clip works
//                 if let Some(platform) = &options.platform {
//                     if platform.to_lowercase() == "android" {
//                         item.r#type = "clip".to_string();
//                     }
//                 }
//             }
//         }
//     }
// }

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
        //item.identifier = Some("tv.plex.provider.discover".to_string());
        // let mut library_section_id: Vec<Option<u32>> = vec![]; //librarySectionID
        for mut hub in item.children_mut() {
            // we only process collection hubs
            if !hub.is_collection_hub() {
                new_hubs.push(hub.to_owned());
                continue;
            }

            if !config.include_watched {
                hub.children_mut().retain(|x| !x.is_watched());
            }
            //hub.context = Some("hub.home.watchlist_available".to_string());
            //hub.r#type = "clip".to_string();
            // hub.placeholder = Some(SpecialBool::new(true));
            //hub.placeholder = Some(true);

            let p = new_hubs.iter().position(|v| v.title == hub.title);
            // if hub.r#type != "clip" {
            //     hub.r#type = "mixed".to_string();
            // }
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

#[derive(Default, Debug, Clone)]
pub struct LibraryMixTransform {
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
impl Transform for LibraryMixTransform {
    async fn transform_mediacontainer(
        &self,
        item: &mut MediaContainer,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
        let config: Config = Config::figment().extract().unwrap();
        let mut children: Vec<MetaData> = vec![];
        let mut total_size_including_watched = 0;

        for id in self.collection_ids.clone() {
            let mut c = plex_client
                .clone()
                .get_cached(
                    plex_client.get_collection_children(
                        id,
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

            total_size_including_watched +=
                c.media_container.total_size.unwrap();
            if !config.include_watched {
                c.media_container.children_mut().retain(|x| !x.is_watched());
            }

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
        if !config.include_watched {
            item.total_size = Some(children.len() as i32);
        } else {
            item.total_size = Some(total_size_including_watched);
        };
        // always metadata
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
pub struct HubStyleTransform {
    pub is_home: bool, // use clip instead of hero for android
}

#[async_trait]
impl Transform for HubStyleTransform {
    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
        let config: Config = Config::figment().extract().unwrap();
        let style = item.style.clone().unwrap_or("".to_string()).to_owned();

        if item.is_collection_hub() {
            let is_hero = item.is_hero(plex_client.clone()).await.unwrap();
            if is_hero {
                item.style = Some("hero".to_string());

                // if options.platform.unwrap_or_default().to_lowercase() == "android"
                //     && self.is_home && options.product.unwrap_or_default().to_lowercase() == "plex for android (mobile)"
                // {
                //     item.r#type = "clip".to_string();
                // }
                item.r#type = "clip".to_string();
                item.meta = Some(Meta {
                    r#type: None,
                    display_fields: vec![DisplayField {
                        r#type: Some("clip".to_string()),
                        fields: vec![
                            "title".to_string(),
                            "parentTitle".to_string(),
                            "originallyAvailableAt".to_string(),
                        ],
                    }],
                    display_images: vec![DisplayImage {
                        r#type: Some("clip".to_string()),
                        image_type: Some("coverArt".to_string()),
                    }],
                });
                let mut futures = FuturesOrdered::new();
                // let now = Instant::now();

                for mut child in item.children() {
                    child.r#type = "clip".to_string();
                    // child.images = vec![Image {
                    //     r#type: "coverArt".to_string(),
                    //     url: "https://image.tmdb.org/t/p/original/3aQb80osBLrUISe6TJ7Y4A0VZJV.jpg".to_string(),
                    //     alt: "Test".to_string()
                    // }];
                    // let style = item.style.clone().unwrap();
                    let client = plex_client.clone();
                    futures.push_back(async move {
                        let mut c = child.clone();

                        let art = child.get_hero_art(client).await;
                        if art.is_some() {
                            // c.art = art.clone();
                            c.images = vec![Image {
                                r#type: "coverArt".to_string(),
                                url: art.clone().unwrap(),
                                alt: "Test".to_string(),
                            }];
                        }
                        // big screen uses thumbs for artwork.... while mobile uses the artwork. yeah...
                        // c.thumb = c.art.clone();
                        c
                    });
                }
                // let elapsed = now.elapsed();
                // println!("Elapsed: {:.2?}", elapsed);
                // let now = Instant::now();

                let children: Vec<MetaData> = futures.collect().await;
                item.set_children(children);
            }
        }
    }
}

/// Collections can be called from hubs as a refresh. But also standalone.
/// We need to know if if its hub called and if the hub is hero styled for media.
#[derive(Default, Debug)]
pub struct CollecionArtTransform {
    pub collection_ids: Vec<u32>,
    pub hub: bool, // if collections is meant for hubs
}

#[async_trait]
impl Transform for CollecionArtTransform {
    async fn transform_mediacontainer(
        &self,
        item: &mut MediaContainer,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
        let mut collection_details = plex_client
            .clone()
            .get_cached(
                plex_client.get_collection(self.collection_ids[0] as i32),
                format!("collection:{}", self.collection_ids[0].to_string()),
            )
            .await;

        if collection_details.is_ok()
            && collection_details
                .unwrap()
                .media_container
                .children()
                .get(0)
                .unwrap()
                .has_label("REPLEXHERO".to_string())
        {
            let mut futures = FuturesOrdered::new();
            // let now = Instant::now();

            for child in item.children() {
                // let style = item.style.clone().unwrap();
                let client = plex_client.clone();
                futures.push_back(async move {
                    let mut c = child.clone();

                    let art = child.get_hero_art(client).await;
                    if art.is_some() {
                        c.art = art.clone();
                    }
                    // big screen uses thumbs for artwork.... while mobile uses the artwork. yeah...
                    // c.thumb = c.art.clone();
                    c
                });
            }

            let children: Vec<MetaData> = futures.collect().await;
            item.set_children(children);
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

#[derive(Default, Debug)]
pub struct HubKeyTransform;

/// We point to replex so we can do some transform on the children calls
#[async_trait]
impl Transform for HubKeyTransform {
    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
        if item.is_collection_hub() {
            if !item.key.contains("replex") {
                // might already been set by the mixings
                item.key = format!("/replex{}", item.key);
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct UserStateTransform;

#[async_trait]
impl Transform for UserStateTransform {
    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
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

#[derive(Default, Debug)]
pub struct TestTransform;

#[async_trait]
impl Transform for TestTransform {
    async fn transform_mediacontainer(
        &self,
        item: &mut MediaContainer,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
        for i in item.children_mut() {
            for x in i.children_mut() {
                x.guids = vec![];
            }
        }
    }
}
