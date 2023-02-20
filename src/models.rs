use crate::plex_client::PlexClient;
use crate::proxy::*;
use crate::utils::*;
use crate::xml::*;
use anyhow::Result;
use async_trait::async_trait;
use axum::headers::ContentType as HContentType;
use axum::{
    body::HttpBody,
    response::{IntoResponse, Response},
    Json,
};
use cached::proc_macro::cached;
use hyper::Body;
use hyper::client::HttpConnector;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::debug;
use yaserde::YaDeserialize;
use yaserde::YaSerialize;
// use parse_display::{Display, FromStr};
// use yaserde_derive::{YaDeserialize, YaSerialize};


#[derive(
    Debug,
    Clone,
)]
pub struct App {
    proxy: Proxy,
    plex: PlexClient
}

pub type HttpClient = hyper::client::Client<HttpConnector, Body>;

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
pub struct MetaData {
    #[yaserde(attribute)]
    #[yaserde(rename = "ratingKey")]
    pub rating_key: Option<String>,
    #[yaserde(attribute)]
    pub key: String,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guid: Option<String>,
    #[yaserde(attribute)]
    pub title: String,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub art: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "parentKey")]
    pub parent_key: Option<String>,
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
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(attribute)]
    pub style: Option<String>,
    // pub context: String,
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
}

impl MetaData {
    pub fn set_children(&mut self, value: Vec<MetaData>) {
        if !self.metadata.is_empty() {
            self.metadata = value;
        } else if !self.directory.is_empty() {
            self.directory = value;
        } else if !self.video.is_empty() {
            self.video = value;
        };
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

#[derive(
    Debug, Serialize, Deserialize, Clone, PartialEq, Eq, YaDeserialize, YaSerialize, Default,
)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
// #[display("{key}")]
pub struct Hub {
    #[yaserde(attribute)]
    pub key: String,
    #[yaserde(attribute)]
    #[yaserde(rename = "hubKey")]
    pub hub_key: Option<String>,
    #[yaserde(attribute)]
    pub title: String,
    #[yaserde(attribute)]
    #[yaserde(rename = "hubIdentifier")]
    pub hub_identifier: String,
    #[yaserde(attribute)]
    pub context: String,
    #[yaserde(attribute)]
    #[yaserde(rename = "type")]
    pub r#type: String,
    #[yaserde(attribute)]
    pub size: i32,
    #[yaserde(attribute)]
    pub more: bool,
    #[yaserde(attribute)]
    pub style: String,
    #[yaserde(attribute)]
    pub promoted: Option<bool>,
    #[serde(rename = "Metadata", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Metadata")]
    pub metadata: Vec<MetaData>,
    #[serde(rename = "Directory", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Directory")]
    directory: Vec<MetaData>, // only avaiable in XML
    #[serde(rename = "Video", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Video")]
    video: Vec<MetaData>, // again only xml, but its the same as directory and metadata
}

impl Hub {
    // as there are 3 diff names for it
    pub fn set_children(&mut self, value: Vec<MetaData>) {
        if !self.metadata.is_empty() {
            self.metadata = value;
        } else if !self.directory.is_empty() {
            self.directory = value;
        } else if !self.video.is_empty() {
            self.video = value;
        };
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

#[derive(
    Debug, Serialize, Deserialize, Clone, PartialEq, Eq, YaDeserialize, YaSerialize, Default,
)]
// #[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
#[yaserde(root = "MediaContainer")]
pub struct MediaContainer {
    #[yaserde(attribute)]
    pub size: Option<i32>,
    #[yaserde(attribute)]
    #[yaserde(rename = "allowSync")]
    pub allow_sync: Option<bool>,
    #[yaserde(attribute)]
    pub identifier: Option<String>,
    #[serde(rename = "librarySectionID")]
    #[yaserde(rename = "librarySectionID")]
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
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
    // pub hub: Vec<Hub>,
    pub hub: Vec<MetaData>,
    #[serde(rename = "Metadata", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub metadata: Vec<MetaData>,
    #[serde(rename = "Video", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Video")]
    pub video: Vec<MetaData>, // again only xml, but its the same as directory and metadata
}

impl MediaContainer {
    pub fn set_type(&mut self, value: String) {
        for hub in &mut self.hub {
            hub.r#type = value.clone();
        }
    }
    pub fn set_children(&mut self, value: Vec<MetaData>) {
        if !self.metadata.is_empty() {
            self.metadata = value;
        } else if !self.hub.is_empty() {
            self.hub = value;
        } else if !self.video.is_empty() {
            self.video = value;
        };
    }

    pub fn children(&mut self) -> Vec<MetaData> {
        if !self.metadata.is_empty() {
            return self.metadata.clone();
        } else if !self.hub.is_empty() {
            return self.hub.clone();
        } else if !self.video.is_empty() {
            return self.video.clone();
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

// TODO: Merge hub keys when mixed
fn merge_children_keys(mut key_left: String, mut key_right: String) -> String {
    key_left = key_left.replace("/hubs/library/collections/", "");
    key_left = key_left.replace("/library/collections/", "");
    key_left = key_left.replace("/children", "");
    key_right = key_right.replace("/hubs/library/collections/", "");
    key_right = key_right.replace("/library/collections/", "");
    key_right = key_right.replace("/children", "");

    format!("/hubs/library/collections/{},{}/children", key_right, key_left)
}

impl MediaContainerWrapper<MediaContainer> {
    // pub fn set_content_type(&mut self, content_type: &http::HeaderValue) {
    //     // let b = HContentType::json();
    //     if content_type == HContentType::json() {
    //         self.content_type = ContentType::Json
    //     } else {
    //         self.content_type = ContentType::Xml
    //     }
    // }
    pub fn make_mixed(mut self) -> Self {
        // if !self.metadata.is_empty() {
        //     let collections = self.media_container.hub;
        // } else if !self.hub.is_empty() {
        // }
        // let collections = self.media_container.get_children();
        
        let collections = self.media_container.children();
        let mut new_collections: Vec<MetaData> = vec![];
        for mut hub in collections {
            // if hub.context.is_none() { // not an collection or
            //     continue
            // }
            let p = new_collections.iter().position(|v| v.title == hub.title);
            hub.r#type = "mixed".to_string();
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
        self.media_container.library_section_id = None;
        self.media_container.library_section_title = None;
        self.media_container.library_section_uuid = None;
        self.media_container.size = Some(size.try_into().unwrap());
        // trace!("mangled promoted container {:#?}", container);
        self.media_container.set_children(new_collections);
        self
    }

    // pub async fn fix_permissions(&mut self, plex: PlexClient) -> &Self {
    pub async fn fix_permissions(&mut self, plex: PlexClient) -> Self {
        debug!("Fixing hub permissions");
        let collections = self.media_container.children();
        let mut custom_collections: Vec<MetaData> = vec![];
        let mut processed_section_ids: Vec<u32> = vec![];

        for hub in collections.clone() {
            // dbg!(&hub);
            // dbg!("yp");
            if hub.metadata.is_empty() {
                // debug!("metadata is empty");
                continue;
            }
            let section_id = hub.metadata[0].library_section_id.unwrap();
            if processed_section_ids.contains(&section_id) {
                continue;
            }

            processed_section_ids.push(section_id);
            // TODO: Use join to join these async requests
            let mut c = plex.get_section_collections(section_id).await.unwrap();
            // dbg!(&c);
            custom_collections.append(&mut c);
        }
        // dbg!(&custom_collections);

        let custom_collections_keys: Vec<String> =
            custom_collections.iter().map(|c| c.key.clone()).collect();

        // let slice = &collections[..];
        let new_collections: Vec<MetaData> = collections
            .into_iter()
            .filter(|c| {
                c.context.clone().unwrap() != "hub.custom.collection"
                    || custom_collections_keys.contains(&c.key)
            })
            .collect();

        // println!("{:#?}", new_collections.len());
        let mut new = self.clone();
        let size = new_collections.len();
        new.media_container.hub = new_collections; // uch need to know if this is a hub or not
        new.media_container.size = Some(size.try_into().unwrap());
        new
    }

    // pub fn merge_children(mut self, children) -> Self {

    // }
}

async fn get_collections() -> Result<Vec<MetaData>> {
    // let plex_client = create_client_from_request(&req).unwrap();
    // let plex_api = plex_api::Server::new("http://100.91.35.113:32400", plex_client).await.unwrap();
    // let mut collections = vec![];
    // let api = self.plex_api.clone().unwrap();
    // for library in api.libraries() {
    //     // library.media

    //     let mut resp: MediaContainerWrapper<MediaContainer> = api
    //         .client()
    //         .get(format!("/library/sections/{}/collections", library.id()))
    //         .json()
    //         .await?;
    //     collections.append(&mut resp.media_container.metadata);
    // }
    // println!("no cache");
    // Ok(MediaContainerWrapper::default().media_container.metadata)
    Ok(vec![])
}

#[cached(
    time = 720,
    // key = "String",
    // convert = r#"{ proxy.get_x_plex_token() }"#
)]
pub async fn get_cached_collections() -> Vec<MetaData> {
    get_collections().await.unwrap()
}

// async fn get_collections() -> Result<Vec<MetaData>> {
//     // let plex_client = create_client_from_request(&req).unwrap();
//     // let plex_api = plex_api::Server::new("http://100.91.35.113:32400", plex_client).await.unwrap();
//     let mut collections = vec![];
//     let api = self.plex_api.clone().unwrap();
//     for library in api.libraries() {
//         // library.media

//         let mut resp: MediaContainerWrapper<MediaContainer> = api
//             .client()
//             .get(format!("/library/sections/{}/collections", library.id()))
//             .json()
//             .await?;
//         collections.append(&mut resp.media_container.metadata);
//     }
//     // println!("no cache");
//     Ok(collections)
// }

// async fn mangle_hubs_permissions(
//     mut container: MediaContainerWrapper<MediaContainer>,
//     server: &plex_api::Server,
// ) -> Result<MediaContainerWrapper<MediaContainer>> {
//     // if container.media_container.hub.is_none() {
//     //     // nothing todo
//     //     return container;
//     // }

//     // TODO: Use get and set children
//     let collections = container.media_container.hub;
//     // println!("{:#?}", hub_collections.len());

//     let custom_collections = get_cached_collections(&server).await;

//     let custom_collections_keys: Vec<String> =
//         custom_collections.iter().map(|c| c.key.clone()).collect();

//     let new_collections: Vec<Hub> = collections
//         .into_iter()
//         .filter(|c| {
//             c.context != "hub.custom.collection"
//                 || custom_collections_keys.contains(&c.key)
//         })
//         .collect();

//     // println!("{:#?}", new_collections.len());

//     let size = new_collections.len();
//     container.media_container.hub = new_collections;
//     container.media_container.size = Some(size.try_into().unwrap());
//     Ok(container)
// }

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

// pub trait FromResponse {
//     /// Init self
//     fn from_response(self) -> Self;
// }

// impl FromResponse for Response<Body>
// {
//     fn into_response(self) -> Self {
//         // Self {

//         // }
//     }
// }

// impl MediaContainerWrapper<T> {
// impl<T: Display> MediaContainerWrapper<T> {
//     fn from_response(&self) {

//     }
// }

// impl Default for MediaContainerWrapper<T> {
//     fn default() -> Self {media_container: T,  content_type: ContentType::Xml}
// }
