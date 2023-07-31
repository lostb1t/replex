use std::sync::Arc;

// use crate::models::*;
use typed_builder::TypedBuilder;


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
    pub transforms: Vec<Arc<dyn Transform>>, // <--- RUST WANTS A TYPE FOR ITEM, BUT DONT KNOW IT AS IT IS SET IN THE TRAIT IMPLEMENTATION
}

impl TransformBuilder {
    #[inline]
    pub fn new() -> Self {
        Self {
            transforms: Vec::new(),
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
        dbg!("do something");
    }
}

// example usage

// metadata = MetaData {
//     id: 34
// }
// transform = TransformBuilder::builder().transforms(CollectionPermissionsTransform::new());