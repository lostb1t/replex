use async_trait::async_trait;
use std::sync::Arc;
// use crate::models::*;
use crate::{models::*, plex_client::PlexClient, utils::*};
use typed_builder::TypedBuilder;

// pub struct MetaDataTest {
//     pub id: i32
// }

#[async_trait]
pub trait Transform: Send + Sync + 'static {
    // type Item;
    async fn transform(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    );
}

// #[derive(TypedBuilder)]
// #[derive(Default)]
#[derive(Clone)]
pub struct TransformBuilder {
    pub plex_client: PlexClient,
    pub options: PlexParams,
    pub transforms: Vec<Arc<dyn Transform>>,
}

impl TransformBuilder {
    #[inline]
    pub fn new(plex_client: PlexClient, options: PlexParams) -> Self {
        Self {
            transforms: Vec::new(),
            plex_client,
            options,
        }
    }

    #[inline]
    pub fn with_transform<T: Transform>(mut self, transform: T) -> Self {
        self.transforms.push(Arc::new(transform));
        self
    }

    // #[inline]
    // pub fn apply_to(mut self, contaoner: MediaContainerWrapper) -> Self {
    //     self.transforms.push(Arc::new(transform));
    //     self
    // }

    pub async fn apply_to(
        self,
        container: &mut MediaContainerWrapper<MediaContainer>,
    ) {
        for item in container.media_container.test() { // TODO: join all futures
            for t in self.transforms.clone() {
                // dbg!("yo");
                t.transform(
                    item,
                    self.plex_client.clone(),
                    self.options.clone(),
                )
                .await;
            }
        }
    }
}

// #[derive(Default)]
// pub struct CollectionPermissionTransform;

// impl Transform for CollectionPermissionTransform {
//     // type Item = MetaData;
//     fn transform(&self, item: &mut MetaData, plex_client: PlexClient) {
//         // dbg!("do something");
//         // tracing::debug!("sup");
//     }
// }

#[derive(Default, Debug)]
pub struct StyleTransform;

#[async_trait]
impl Transform for StyleTransform {
    // type Item = MetaData;
    async fn transform(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexParams,
    ) {
        
        if item.is_collection_hub() {
            let mut collection_details = plex_client
                .get_collection(get_collection_id_from_child_path(
                    item.key.clone(),
                ))
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

// example usage

// metadata = MetaData {
//     id: 34
// }
// transform = TransformBuilder::builder().transforms(CollectionPermissionsTransform::new());
