use serde::{Deserialize, Serialize};
use tracing::{debug};
use axum::{
    body::HttpBody,
    response::{IntoResponse, Response},
    Json,
};
use itertools::Itertools;

use crate::utils::*;
use crate::xml::*;
use crate::proxy::*;
use yaserde::YaSerialize;
use yaserde::YaDeserialize;
// use parse_display::{Display, FromStr};
// use yaserde_derive::{YaDeserialize, YaSerialize};

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
    pub rating_key: String,
    #[yaserde(attribute)]
    pub key: String,
    #[yaserde(attribute)]
    pub guid: String,
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
    pub library_section_id: Option<u32>,
    #[yaserde(attribute)]
    #[yaserde(rename = "librarySectionTitle")]
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

    pub fn get_children(&mut self) -> Vec<MetaData> {
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
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Eq,
    YaDeserialize,
    YaSerialize,
    Default,
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
    pub hub: Vec<Hub>,
    #[serde(rename = "Metadata", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub metadata: Vec<MetaData>,
}

impl MediaContainer {
    pub fn set_type(&mut self, value: String) {
        for hub in &mut self.hub {
            hub.r#type = value.clone();
        }
    }
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
    pub content_type: ContentType
}

impl MediaContainerWrapper<MediaContainer> {

    pub async fn mangle_hubs_promoted(
        mut self
    ) -> Self {
        // TODO: Dont make this hardcoded just get the first value of pinnedContentDirectoryID
        // let mut container: MediaContainerWrapper<MediaContainer> =
        //     MediaContainerWrapper::default();

        // if content_directory_id == "1" {
        //     container = get_promoted_hubs(client_ip, req).await?;
        // }
    
        // for hub in &container.media_container.hub {
        //     for item in hub.metadata {
        //         dbg!(item);
        //     }
        //     // dbg!(hub);
        // }
    
        let collections = self.media_container.hub;
        // let new_collections: Vec<Hub> = collections.clone();
        let mut new_collections: Vec<Hub> = vec![];
        // let movies: Vec<Hub> = collections
        //     .iter()
        //     .filter(|c| {
        //         c.r#type != "movie"
        //     })
        //     .cloned().collect();
        // let shows: Vec<Hub> = collections
        //     .iter()
        //     .filter(|c| {
        //         c.r#type != "movie"
        //     })
        //     .cloned().collect();
    
        for mut hub in collections {
            // if hub.r#type == "movie":
            // let c = new_collections.iter().filter(|v| v.title == hub.title);
            let p = new_collections.iter().position(|v| v.title == hub.title);
            // if Some(p) {
            //     new_collections[p]
            // }
            hub.r#type = "mixed".to_string();
            match p {
                //Some(v) => new_collections[v].metadata.extend(hub.metadata),
                // Some(v) => {
                //     let c = new_collections[v].metadata.clone();
                //     let h = hub.metadata;
                //     new_collections[v].metadata = c.into_iter().merge(h.into_iter()).collect::<Vec<MetaData>>();
                // }
                // Some(v) => {
                //     // let c = new_collections[v].metadata.clone();
                //     // let h = hub.metadata;
                //     new_collections[v].metadata = vec![new_collections[v].metadata.clone(), hub.metadata].into_iter().kmerge().collect::<Vec<MetaData>>();
                // }
                Some(v) => {
                    let c = new_collections[v].get_children();
                    // let h = hub.metadata;
                    new_collections[v].set_children(
                        c.into_iter()
                            .interleave(hub.get_children())
                            .collect::<Vec<MetaData>>(),
                    );
                }
                None => new_collections.push(hub),
            }
            // for item in hub.metadata {
            //     dbg!(item);
            // }
        }
        //container.media_container.set_type("mixed".to_owned());
        // dbg!(&movies);
        // lets get everything into
        // collections = collections.set_metadata_type("mixed".to_owned());
    
        // container.media_container.hub = mangled_collections;
        let size = new_collections.len();
        //container.media_container.hub = movies;
        // container.media_container.library_section_id = Some("home".to_string());
        self.media_container.library_section_id = None;
        self.media_container.library_section_title = None;
        self.media_container.library_section_uuid = None;
        self.media_container.size = Some(size.try_into().unwrap());
        // trace!("mangled promoted container {:#?}", container);
        self.media_container.hub = new_collections;
        self
    }

    pub async fn fix_permissions(mut self, proxy: &Proxy) -> Self {
        debug!("Fixing hub permissions");
        let collections = self.media_container.hub;
        // println!("{:#?}", hub_collections.len());
    
        let custom_collections = get_cached_collections(proxy).await;
    
        let custom_collections_keys: Vec<String> =
            custom_collections.iter().map(|c| c.key.clone()).collect();
    
        let new_collections: Vec<Hub> = collections
            .into_iter()
            .filter(|c| {
                c.context != "hub.custom.collection"
                    || custom_collections_keys.contains(&c.key)
            })
            .collect();
    
        // println!("{:#?}", new_collections.len());
    
        let size = new_collections.len();
        self.media_container.hub = new_collections;
        self.media_container.size = Some(size.try_into().unwrap());
        self       
    }
}

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

