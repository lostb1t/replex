pub mod collection_style;
pub mod hub_interleave;
pub mod hub_watched;
pub mod user_state;
pub mod hub_key;
pub mod hub_limit;
pub mod media_style;
pub mod hub_section_directory;
pub mod hub_style;
pub mod library_interleave;
pub mod restriction_filter;

pub use collection_style::CollectionStyleTransform;
pub use hub_interleave::HubInterleaveTransform;
pub use user_state::UserStateTransform;
pub use hub_watched::HubWatchedTransform;
pub use hub_key::HubKeyTransform;
pub use hub_limit::HubChildrenLimitTransform;
pub use media_style::MediaStyleTransform;
pub use hub_section_directory::HubSectionDirectoryTransform;
pub use hub_style::{ClientHeroStyle, HubStyleTransform};
pub use library_interleave::LibraryInterleaveTransform;
pub use restriction_filter::HubRestrictionFilter;

use crate::{
    models::*,
    plex_client::{PlexClient},
};

use async_trait::async_trait;
use async_recursion::async_recursion;
use futures_util::{
    future::{self},
    StreamExt,
};
use std::sync::Arc;

#[async_trait]
pub trait Transform: Send + Sync + 'static {
    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexContext,
    ) {
    }
    async fn transform_mediacontainer(
        &self,
        item: MediaContainer,
        plex_client: PlexClient,
        options: PlexContext,
    ) -> MediaContainer {
        item
    }
}

#[async_trait]
pub trait Filter: Send + Sync + 'static {
    async fn filter_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexContext,
    ) -> bool {
        true
    }
    async fn filter_mediacontainer(
        &self,
        item: MediaContainer,
        plex_client: PlexClient,
        options: PlexContext,
    ) -> bool {
        true
    }
}

// #[derive(TypedBuilder)]
// #[derive(Default)]
#[derive(Clone)]
pub struct TransformBuilder {
    pub plex_client: PlexClient,
    pub options: PlexContext,
    pub mix: bool,
    pub transforms: Vec<Arc<dyn Transform>>,
    pub filters: Vec<Arc<dyn Filter>>,
}

impl TransformBuilder {
    #[inline]
    pub fn new(plex_client: PlexClient, options: PlexContext) -> Self {
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
        'outer: for item in metadata {
            for filter in self.filters.clone() {
                // dbg!("filtering");
                if filter
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
                let childs = self.apply_to_metadata(item.children_mut()).await;
                item.set_children(childs);
            }

            filtered_childs.push(item.to_owned());
        }

        return filtered_childs;
    }

    pub async fn apply_to(
        self,
        container: &mut MediaContainerWrapper<MediaContainer>,
    ) {
        let children = container.media_container.children_mut();
        let new_children = self.apply_to_metadata(children).await;
        container.media_container.set_children(new_children);
        
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
            //for k in container.media_container.children_mut()

            // dont use join as it needs ti be executed in order
            // let l = std::cell::RefCell::new(&mut container.media_container);
            container.media_container = t
                .transform_mediacontainer(
                    container.media_container.clone(),
                    self.plex_client.clone(),
                    self.options.clone(),
                )
                .await;
            // dbg!(container.media_container.size);
        }

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
    
    pub async fn apply_to_test(
        self,
        container: &mut MediaContainerWrapper<MediaContainer>,
    ) {
        for t in self.transforms.clone() {
      
            for child in container.media_container.children_mut() {
               t.transform_metadata(
                            child,
                            self.plex_client.clone(),
                            self.options.clone(),
                        ).await;
            }
            //future::join_all(futures).await;

            // dont use join as it needs ti be executed in order
            // let l = std::cell::RefCell::new(&mut container.media_container);
            container.media_container = t
                .transform_mediacontainer(
                    container.media_container.clone(),
                    self.plex_client.clone(),
                    self.options.clone(),
                )
                .await;
            // dbg!(container.media_container.size);
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

pub fn hero_meta() -> Meta {
    Meta {
        r#type: None,
        // r#type: Some(MetaType {
        //     active: Some(true),
        //     r#type: Some("mixed".to_string()),
        //     title: Some("All".to_string()),
        // }),
        // style: Some(Style::Hero.to_string().to_lowercase()),
        // display_fields: vec![],
        display_fields: vec![
            DisplayField {
                r#type: Some("movie".to_string()),
                fields: vec![
                    "title".to_string(),
                    "originallyAvailableAt".to_string(),
                ],
            },
            DisplayField {
                r#type: Some("show".to_string()),
                fields: vec![
                    "title".to_string(),
                    "originallyAvailableAt".to_string(),
                ],
            },
            DisplayField {
                r#type: Some("clip".to_string()),
                fields: vec![
                    "title".to_string(),
                    "originallyAvailableAt".to_string(),
                ],
            },
            DisplayField {
                r#type: Some("mixed".to_string()),
                fields: vec![
                    "title".to_string(),
                    "originallyAvailableAt".to_string(),
                ],
            },
        ],
        display_images: vec![
            DisplayImage {
                r#type: Some("hero".to_string()),
                image_type: Some("coverArt".to_string()),
            },
            DisplayImage {
                r#type: Some("mixed".to_string()),
                image_type: Some("coverArt".to_string()),
            },
            DisplayImage {
                r#type: Some("clip".to_string()),
                image_type: Some("coverArt".to_string()),
            },
            DisplayImage {
                r#type: Some("movie".to_string()),
                image_type: Some("coverArt".to_string()),
            },
            DisplayImage {
                r#type: Some("show".to_string()),
                image_type: Some("coverArt".to_string()),
            },
        ],
    }
}
