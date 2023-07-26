use std::fmt::Display;
use std::io::Read;
use std::str::FromStr;

use crate::config::*;
use crate::plex_client::PlexClient;
use crate::proxy::*;
use crate::utils::*;
use crate::xml::*;
use anyhow::Result;
use async_trait::async_trait;
use axum::{
    body::Body,
    response::{IntoResponse, Response},
    Json,
};
use hyper::client::HttpConnector;
use serde_aux::prelude::{
    deserialize_number_from_string, deserialize_option_number_from_string,
    deserialize_string_from_number,
};
// use hyper::Body;
use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::serde_as;
use tracing::debug;
use yaserde::YaDeserialize;
use yaserde::YaSerialize;
// use replex::settings::*;
//mod replex;

// use parse_display::{Display, FromStr};
// use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Debug, Clone, Default)]
pub struct ReplexOptions {
    pub limit: Option<i32>,
}

// impl Default for ReplexOptions {
//     fn default() -> Self { limit: None }
// }

#[derive(Debug, Clone)]
pub struct App {
    proxy: Proxy,
    plex: PlexClient,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Eq,
    YaDeserialize,
    YaSerialize,
    Default,
    PartialOrd,
)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct Label {
    #[yaserde(attribute)]
    id: i32,
    #[yaserde(attribute)]
    tag: String,
    #[yaserde(attribute)]
    filter: String,
}

pub type HttpClient = hyper::client::Client<HttpConnector, Body>;

#[derive(Debug, Serialize, Deserialize, Clone, YaDeserialize, YaSerialize)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
#[serde_as]
pub struct MetaData {
    #[yaserde(attribute)]
    #[yaserde(rename = "ratingKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating_key: Option<String>,
    #[yaserde(attribute)]
    pub key: String,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guid: Option<String>,
    #[yaserde(attribute)]
    // #[yaserde(skip_serializing = true)]
    // #[serde(skip_serializing)]
    pub title: String,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagline: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composite: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_group: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_mode: Option<u32>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub art: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "parentKey")]
    pub parent_key: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "parentRatingKey")]
    pub parent_rating_key: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "parentTitle")]
    pub parent_title: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "grandparentRatingKey")]
    pub grandparent_rating_key: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "grandparentKey")]
    pub grandparent_key: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "grandparentGuid")]
    pub grandparent_guid: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "grandparentTitle")]
    pub grandparent_title: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "grandparentThumb")]
    pub grandparent_thumb: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "grandparentArt")]
    pub grandparent_art: Option<String>,
    #[yaserde(attribute)]
    #[yaserde(rename = "type")]
    #[serde(rename = "librarySectionID")]
    #[yaserde(rename = "librarySectionID")]
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library_section_id: Option<u32>,
    #[yaserde(attribute)]
    #[yaserde(rename = "librarySectionTitle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library_section_title: Option<String>,
    #[yaserde(attribute)]
    #[yaserde(rename = "librarySectionKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library_section_key: Option<String>,
    #[yaserde(rename = "type")]
    #[yaserde(attribute)]
    pub r#type: String,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promoted: Option<bool>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[yaserde(attribute)]
    #[yaserde(rename = "hubKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hub_key: Option<String>,
    #[yaserde(attribute)]
    #[yaserde(rename = "hubIdentifier")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hub_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(attribute)]
    //#[serde(deserialize_with = "str_or_i32")]
    pub size: Option<i32>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(attribute)]
    pub style: Option<String>,
    #[yaserde(skip)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    #[serde(rename = "Metadata", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Metadata")]
    pub metadata: Vec<MetaData>,
    #[serde(rename = "Directory", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Directory")]
    pub directory: Vec<MetaData>, // only avaiable in XML
    #[serde(rename = "Video", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Video")]
    pub video: Vec<MetaData>, // again only xml, but its the same as directory and metadata
    #[yaserde(attribute, rename = "childCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_option_string_from_number")]
    pub child_count: Option<String>,
    #[yaserde(attribute)]
    #[yaserde(rename = "skipChildren")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_children: Option<bool>,
    #[yaserde(attribute)]
    #[yaserde(rename = "leafCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leaf_count: Option<i32>,
    #[yaserde(attribute)]
    #[yaserde(rename = "viewedLeafCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewed_leaf_count: Option<i32>,
    #[yaserde(attribute)]
    #[yaserde(rename = "viewCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_count: Option<i32>,
    #[serde(rename = "Label", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    // #[yaserde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Label", default)]
    // #[yaserde(flatten)]
    #[yaserde(child)]
    pub labels: Vec<Label>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub originally_available_at: Option<String>,
    // #[yaserde( attribute)]
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub rating: Option<f64>,
    // #[yaserde(rename = "audienceRating" attribute)]
    // #[serde(rename = "audienceRating", skip_serializing_if = "Option::is_none")]
    // pub audience_rating: Option<f64>,
    // #[yaserde(attribute)]
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub index: Option<i32>,
    // #[yaserde(rename = "primaryExtraKey", attribute)]
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub primary_extra_key: Option<String>,
}

// impl YaDeserialize for MetaData {
//     fn deserialize<R: Read>(reader: &mut yaserde::de::Deserializer<R>) -> Result<Self, String> {
//       // deserializer code
//     }
//   }

pub(crate) fn deserialize_option_string_from_number<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(deserialize_string_from_number(deserializer)?))
}

impl MetaData {
    pub fn is_hub(&self) -> bool {
        self.hub_identifier.is_some()
    }

    pub fn is_media(&self) -> bool {
        !self.is_hub() && (self.r#type == "movie" || self.r#type == "show")
    }

    pub fn is_collection_hub(&self) -> bool {
        self.is_hub()
            && self.context.is_some()
            && self
                .context
                .clone()
                .unwrap()
                .starts_with("hub.custom.collection")
    }

    // pub async fn replex(&mut self, plex: &PlexClient) -> MetaData {
    //     if self.context.clone().unwrap_or_default().starts_with("hub.custom.collection") {
    //         self.r#type = "mixed".to_string();
    //         self.apply_hub_style(&plex).await;
    //     }
    //     self.clone()
    // }

    pub async fn apply_hub_style(&mut self, plex: &PlexClient) {
        if self.is_collection_hub() {
            let mut children: Vec<MetaData> = self.children();
            // dbg!(get_collection_id_from_child_path(self.key.clone()));
            let mut collection_details = plex
                .get_collection(get_collection_id_from_child_path(self.key.clone()))
                .await
                .unwrap(); // TODO: Cache
                           // dbg!("yup");       // dbg!(&collection_details);
            if collection_details
                .media_container
                .children()
                .get(0)
                .unwrap()
                .has_label("REPLEXHERO".to_string())
            {
                self.style = Some("hero".to_string());

                // for android
                self.meta = Some(Meta {
                    // r#type: Some("clip".to_string()),
                    r#type: None,
                    display_fields: vec![
                    DisplayField {
                        r#type: Some("movie".to_string()),
                        fields: vec!["title".to_string(), "year".to_string()],
                    },
                    DisplayField {
                        r#type: Some("show".to_string()),
                        fields: vec!["title".to_string(), "year".to_string()],
                    }],
                });          
                self.r#type = "clip".to_string();
                // children[0].r#type = "clip".to_string();

                // let child = children.get(0).unwrap().clone();
                // child.r#type = "clip".to_string();
                // children.push(children.get(0).unwrap().r#type = "clip".to_string())
            }

            // self.set_children(children);
            // dbg!(collection_details);
        }
        // self
    }

    fn has_label(&self, name: String) -> bool {
        for label in &self.labels {
            if label.tag == name {
                return true;
            }
        }
        false
        // collection_details.media_container.directory.get(0).unwrap().label.is_some()
    }

    fn is_watched(&self) -> bool {
        if self.view_count.is_some() && self.view_count.unwrap_or_default() > 0 {
            return true;
        }
        if self.viewed_leaf_count.is_some() && self.viewed_leaf_count.unwrap_or_default() > 0 {
            return true;
        }
        false
    }

    fn remove_watched(&mut self) {
        let new_children: Vec<MetaData> = self
            .children()
            .into_iter()
            .filter(|c| !c.is_watched())
            .collect::<Vec<MetaData>>();

        let size = new_children.len();
        self.size = Some(size.try_into().unwrap());
        // trace!("mangled promoted container {:#?}", container);
        self.set_children(new_children);
        //self
    }

    // TODO: Does not work when using a new instance
    pub fn set_children(&mut self, value: Vec<MetaData>) {
        let len: i32 = value.len().try_into().unwrap();
        if !self.metadata.is_empty() {
            self.metadata = value;
        } else if !self.directory.is_empty() {
            self.directory = value;
        } else if !self.video.is_empty() {
            self.video = value;
        };
        self.size = Some(len);
    }

    pub fn children(&mut self) -> Vec<MetaData> {
        if !self.metadata.is_empty() {
            return self.metadata.clone();
        } else if !self.directory.is_empty() {
            return self.directory.clone();
        } else if !self.video.is_empty() {
            return self.video.clone();
        };
        vec![]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, YaDeserialize, YaSerialize, Default)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
#[yaserde(root = "MediaContainer")]
pub struct MediaContainer {
    #[yaserde(attribute)]
    //#[serde(deserialize_with = "str_or_i32")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i32>,
    #[yaserde(attribute)]
    #[yaserde(rename = "totalSize")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_size: Option<i32>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    #[yaserde(attribute)]
    #[yaserde(rename = "allowSync")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_sync: Option<bool>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[yaserde(attribute, rename = "librarySectionID")]
    #[serde(rename = "librarySectionID", skip_serializing_if = "Option::is_none")]
    pub library_section_id: Option<u32>,
    #[yaserde(attribute)]
    #[yaserde(rename = "librarySectionTitle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library_section_title: Option<String>,
    #[serde(rename = "librarySectionUUID")]
    #[yaserde(rename = "librarySectionUUID")]
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library_section_uuid: Option<String>,
    #[serde(rename = "Hub", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Hub")]
    pub hub: Vec<MetaData>,
    #[serde(rename = "Metadata", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Metadata")]
    pub metadata: Vec<MetaData>,
    #[serde(rename = "Video", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Video")]
    pub video: Vec<MetaData>, // again only xml, but its the same as directory and metadata
    #[serde(rename = "Directory", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Directory")]
    pub directory: Vec<MetaData>,
}

#[derive(Debug, Serialize, Deserialize, Clone, YaDeserialize, YaSerialize, Default)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct DisplayField {
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "type")]
    pub r#type: Option<String>,
    // #[yaserde(attribute)]
    pub fields: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, YaDeserialize, YaSerialize, Default)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    #[serde(rename = "DisplayFields")]
    pub display_fields: Vec<DisplayField>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "type")]
    pub r#type: Option<String>,
}

// pub fn remove_watched(item: MetaData) {
//     let new_children: Vec<MetaData> = self
//         .children()
//         .into_iter()
//         .filter(|c| !c.is_watched())
//         .collect::<Vec<MetaData>>();

//     // let size = new_children.len();
//     // self.size = Some(size.try_into().unwrap());
//     // // trace!("mangled promoted container {:#?}", container);
//     // self.set_children(new_children);
//     //sel
// }

impl MediaContainer {
    pub fn set_type(&mut self, value: String) {
        for hub in &mut self.hub {
            hub.r#type = value.clone();
        }
    }
    pub fn set_children(&mut self, value: Vec<MetaData>) {
        let len: i32 = value.len().try_into().unwrap();
        if !self.metadata.is_empty() {
            self.metadata = value;
        } else if !self.hub.is_empty() {
            self.hub = value;
        } else if !self.video.is_empty() {
            self.video = value;
        } else if !self.directory.is_empty() {
            self.directory = value;
        };
        self.size = Some(len);
    }

    pub fn children(&mut self) -> Vec<MetaData> {
        if !self.metadata.is_empty() {
            return self.metadata.clone();
        } else if !self.hub.is_empty() {
            return self.hub.clone();
        } else if !self.video.is_empty() {
            return self.video.clone();
        } else if !self.directory.is_empty() {
            return self.directory.clone();
        };
        vec![]
    }
    // pub fn children_type()
}

// impl MediaContainer {
//     fn check_optional_string(&self, value: &Option<Vec<MetaData>>) -> bool {
//         value == &Some("unset".to_string())
//     }
// }

// pub MediaContainerBuilder

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct MediaContainerWrapper<T> {
    #[serde(rename = "MediaContainer")]
    // #[serde(rename="$value")]
    pub media_container: T,
    #[serde(skip_serializing, skip_deserializing)]
    pub content_type: ContentType,
}

#[async_trait]
pub trait FromResponse<T>: Sized {
    async fn from_response(resp: T) -> Result<Self>;
}

// #[async_trait]
// impl<T, R> FromResponse<R> for MediaContainerWrapper<T>
// where
//     T: MediaContainer,
//     R: Response<Body>,
// {
//     async fn from_response(resp: Response<Body>) -> Self {
//         from_response(resp).await.unwrap()
//     }
// }

// pub type Container = MediaContainerWrapper<MediaContainer>;

#[async_trait]
impl FromResponse<Response<Body>> for MediaContainerWrapper<MediaContainer> {
    async fn from_response(resp: Response<Body>) -> Result<MediaContainerWrapper<MediaContainer>> {
        let res = from_response(resp).await?;
        Ok(res)
    }
}

// #[async_trait]
// impl FromResponse for MediaContainerWrapper<MediaContainer> {
//     async fn from_response(resp: Response<Body>) -> Self {
//         from_response(resp).await.unwrap()
//     }
// }

// #[async_trait]
// impl From<Response<Body>> for MediaContainerWrapper<MediaContainer> {
//     async fn from_response(resp: Response<Body>) -> Self {
//         from_response(resp).await.unwrap()
//     }
// }

fn get_collection_id_from_child_path(path: String) -> i32 {
    let mut path = path.replace("/library/collections/", "");
    path = path.replace("/children", "");
    // let id = path.parse();
    // dbg!(&path);
    path.parse().unwrap()
}

// TODO: Merge hub keys when mixed
fn merge_children_keys(mut key_left: String, mut key_right: String) -> String {
    key_left = key_left.replace("/hubs/library/collections/", "");
    key_left = key_left.replace("/library/collections/", "");
    key_left = key_left.replace("/children", "");
    key_right = key_right.replace("/hubs/library/collections/", "");
    key_right = key_right.replace("/library/collections/", "");
    key_right = key_right.replace("/children", "");

    format!(
        "/replex/library/collections/{},{}/children",
        key_right, key_left
    )
}

impl MediaContainerWrapper<MediaContainer> {
    pub fn is_hub(&self) -> bool {
        !self.media_container.hub.is_empty()
    }

    pub fn is_section_hub(&self) -> bool {
        self.is_hub() && self.media_container.library_section_id.is_some()
    }

    /// TODO: use filter an map. And chain them for performance
    pub async fn replex(mut self, plex: PlexClient, options: ReplexOptions) -> Self {
        let config: Config = Config::figment().extract().unwrap();

        // needs to come before process hubs as it will set some valeus to none
        self = self.fix_permissions(&plex).await;

        if self.is_hub() {
            self = self.process_hubs(&plex).await;
        }

        if !config.include_watched {
            self = self.remove_watched();
        }

        if options.limit.is_some() {
            self = self.limit(options.limit.unwrap());
        }

        self
    }

    pub fn limit(mut self, limit: i32) -> Self {
        // let mut children: Vec<MetaData> = vec![];
        // for mut child in self.media_container.children() {
        //     child.truncate(limit);
        //     children.push(child);
        // }
        let len = limit as usize;
        if self.is_hub() {
            let mut hubs: Vec<MetaData> = vec![];
            for mut hub in self.media_container.children() {
                let mut children = hub.children();
                children.truncate(len);
                hub.set_children(children);
                hubs.push(hub);
            }
            self.media_container.set_children(hubs);
        } else {
            let mut children = self.media_container.children();
            children.truncate(len);
            self.media_container.set_children(children);
        }
        self
    }

    // TODO: This should be a trait so we dont repeat ourselfs
    pub fn remove_watched(mut self) -> Self {
        let mut children: Vec<MetaData> = vec![];
        if self.is_hub() {
            for mut child in self.media_container.children() {
                child.remove_watched();
                children.push(child);
            }
        } else {
            children = self
                .media_container
                .children()
                .into_iter()
                .filter(|c| !c.is_watched())
                .collect::<Vec<MetaData>>();
        }
        self.media_container.set_children(children);
        self
    }

    // TODO: Only works for hubs. Make it generic or name it specific for hubs
    pub async fn process_hubs(mut self, plex: &PlexClient) -> Self {
        let collections = self.media_container.children();
        let mut new_collections: Vec<MetaData> = vec![];
        for mut hub in collections {
            if !hub.is_collection_hub() {
                new_collections.push(hub);
                continue;
            }

            hub.apply_hub_style(plex).await;
            if self.is_section_hub() {
                new_collections.push(hub);
                continue;
            }
            let p = new_collections.iter().position(|v| v.title == hub.title);

            if hub.r#type != "clip" {
                hub.r#type = "mixed".to_string();
            }

            match p {
                Some(v) => {
                    new_collections[v].key =
                        merge_children_keys(new_collections[v].key.clone(), hub.key.clone());
                    let c = new_collections[v].children();
                    // let h = hub.metadata;
                    new_collections[v].set_children(
                        c.into_iter()
                            .interleave(hub.children())
                            .collect::<Vec<MetaData>>(),
                    );
                }
                None => new_collections.push(hub),
            }
        }

        let size = new_collections.len();
        self.media_container.size = Some(size.try_into().unwrap());
        self.media_container.set_children(new_collections);
        self
    }

    pub async fn apply_hub_style(&mut self, plex: &PlexClient) -> &Self {
        let mut metadata: Vec<MetaData> = vec![];
        for mut hub in self.media_container.children() {
            if hub.style.is_some() {
                hub.apply_hub_style(plex).await;
                metadata.push(hub);
            }
        }
        self.media_container.set_children(metadata);
        self
    }

    /// collection hubs dont follow plex restrictions.
    /// We fix that by checking the collection endpoint. As that does listen to plex restrictions
    pub async fn fix_permissions(&mut self, plex: &PlexClient) -> Self {
        debug!("Fixing hub permissions");
        let collections = self.media_container.children();
        let mut custom_collections: Vec<MetaData> = vec![];
        let mut processed_section_ids: Vec<u32> = vec![];

        for mut metadata in collections.clone() {
            if metadata.is_hub() && !metadata.is_collection_hub() {
                continue;
            }

            let section_id: u32 = metadata.library_section_id.unwrap_or_else(|| {
                metadata
                    .children()
                    .get(0)
                    .unwrap()
                    .library_section_id
                    .expect("Missing Library section id")
            });

            if processed_section_ids.contains(&section_id) {
                continue;
            }

            processed_section_ids.push(section_id);

            // TODO: Use join to join these async requests
            let mut c = plex.get_section_collections(section_id).await.unwrap();
            custom_collections.append(&mut c);
        }

        let custom_collections_keys: Vec<String> =
            custom_collections.iter().map(|c| c.key.clone()).collect();

        let new_collections: Vec<MetaData> = collections
            .into_iter()
            .filter(|c| !c.is_collection_hub() || custom_collections_keys.contains(&c.key))
            .collect();

        let mut new = self.clone();
        let size = new_collections.len();
        //new.media_container.hub = new_collections; // uch need to know if this is a hub or not
        new.media_container.set_children(new_collections);
        new.media_container.size = Some(size.try_into().unwrap());
        new
    }
}

impl<T> IntoResponse for MediaContainerWrapper<T>
where
    T: Serialize + YaDeserialize + YaSerialize,
{
    fn into_response(self) -> Response {
        match self.content_type {
            ContentType::Json => Json(self).into_response(),
            ContentType::Xml => Xml(self.media_container).into_response(),
        }
    }
}
