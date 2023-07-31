use std::sync::Arc;

// use crate::models::*;
use typed_builder::TypedBuilder;

use crate::plex_client::{PlexClient, self};


pub struct MetaDataTest {
    pub id: i32
}

pub trait Transform: Send + Sync + 'static {
    // type Item;
    fn transform(&self, item: &mut MetaDataTest);
}

// #[derive(TypedBuilder)]
#[derive(Default)]
pub struct TransformBuilder {
    pub plex_client: PlexClient,
    pub transforms: Vec<Arc<dyn Transform>>, // <--- RUST WANTS A TYPE FOR ITEM, BUT DONT KNOW IT AS IT IS SET IN THE TRAIT IMPLEMENTATION
}

impl TransformBuilder {
    #[inline]
    pub fn new(plex_client: PlexClient) -> Self {
        Self {
            transforms: Vec::new(),
            plex_client,
        }
    }

    #[inline]
    pub fn with_transform<T: Transform>(mut self, transform: T) -> Self {
        self.transforms.push(Arc::new(transform));
        self
    }

    pub fn build(self) {
        let m = &mut MetaDataTest {
            id: 34
        };
        for t in self.transforms {
            t.transform(m);
        };
    }
}

#[derive(Default)]
pub struct CollectionPermissionTransform;

impl Transform for CollectionPermissionTransform {
    // type Item = MetaData;
    fn transform(&self, item: &mut MetaDataTest) {
        // dbg!("do something");
        // tracing::debug!("sup");
    }
}

#[derive(Default)]
pub struct StyleTransform;

impl Transform for CollectionPermissionTransform {
    // type Item = MetaData;
    fn transform(&self, item: &mut MetaDataTest, plex_client: PlexClient) {
        // if item.is_collection_hub() {
        //     let mut collection_details = plex_client
        //         .get_collection(get_collection_id_from_child_path(item.key.clone()))
        //         .await
        //         .unwrap(); // TODO: Cache
        //                    // dbg!("yup");       // dbg!(&collection_details);
        //     if collection_details
        //         .media_container
        //         .children()
        //         .get(0)
        //         .unwrap()
        //         .has_label("REPLEXHERO".to_string())
        //     {
        //         self.style = Some("hero".to_string());
        //         // dbg!(&options.platform);
        //         // for android, as it doesnt listen to hero style on home..... so we make it a clip
        //         if let Some(platform) = &options.platform {
        //             if platform.to_lowercase() == "android" {
        //                 // dbg!("We got android");
        //                 // self.meta = Some(Meta {
        //                 //     r#type: None,
        //                 //     display_fields: vec![
        //                 //         DisplayField {
        //                 //             r#type: Some("movie".to_string()),
        //                 //             fields: vec!["title".to_string(), "year".to_string()],
        //                 //         },
        //                 //         DisplayField {
        //                 //             r#type: Some("show".to_string()),
        //                 //             fields: vec!["title".to_string(), "year".to_string()],
        //                 //         },
        //                 //     ],
        //                 // });
        //                 self.r#type = "clip".to_string();
        //             }
        //         }
        //     }
        // }
    }
}


// example usage

// metadata = MetaData {
//     id: 34
// }
// transform = TransformBuilder::builder().transforms(CollectionPermissionsTransform::new());