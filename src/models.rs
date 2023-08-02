use salvo::prelude::*;
use std::str::FromStr;

extern crate mime;
use crate::config::*;
use crate::plex_client::PlexClient;

use crate::utils::*;
use anyhow::Result;
use async_trait::async_trait;
use serde_aux::prelude::deserialize_string_from_number;
// use smartstring::alias::String;
// use hyper::Body;
use itertools::Itertools;
use salvo::http::ReqBody;
use salvo::http::ResBody;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::serde_as;
use tracing::debug;

use salvo::macros::Extractible;
// use replex::settings::*;
//mod replex;

// use parse_display::{Display, FromStr};
// use yaserde_derive::{YaDeserialize, YaSerialize};

pub type HyperRequest = hyper::Request<ReqBody>;
pub type HyperResponse = hyper::Response<ResBody>;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ReplexOptions {
    pub limit: Option<i32>,
    pub platform: Option<String>,
    #[serde(default = "default_as_false")]
    pub include_watched: bool,
}

fn default_as_false() -> bool {
    false
}

#[derive(Serialize, Deserialize, Debug, Extractible, Default, Clone)]
#[salvo(extract(
    default_source(from = "query"),
    default_source(from = "header"),
    rename_all = "camelCase"
))]
pub struct PlexParams {
    #[serde(default, deserialize_with = "deserialize_comma_seperated_string")]
    #[salvo(extract(rename = "contentDirectoryID"))]
    pub content_directory_id: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_comma_seperated_string")]
    #[salvo(extract(rename = "pinnedContentDirectoryID"))]
    pub pinned_content_directory_id: Option<Vec<String>>,
    #[salvo(extract(rename = "X-Plex-Platform"))]
    pub platform: Option<String>,
    pub count: Option<i32>,
    #[salvo(extract(rename = "X-Plex-Client-Identifier"))]
    pub client_identifier: Option<String>,
    #[salvo(extract(rename = "X-Plex-Token"))]
    pub token: Option<String>,
    #[salvo(extract(rename = "X-Plex-Container-Size"))]
    pub container_size: Option<i32>,
    #[salvo(extract(rename = "X-Plex-Container-Start"))]
    pub container_start: Option<i32>,
    // #[salvo(extract(rename = "Accept"))]
    // pub accept: ContentType,
}

pub fn deserialize_comma_seperated_number<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<i32>>, D::Error>
where
    D: Deserializer<'de>,
{
    match Deserialize::deserialize(deserializer)? {
        Some::<String>(s) => {
            let r: Vec<i32> =
                s.split(',').map(|s| s.parse().unwrap()).collect();
            Ok(Some(r))
        }
        None => Ok(None),
    }
}

pub fn deserialize_comma_seperated_string<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    match Deserialize::deserialize(deserializer)? {
        Some::<String>(s) => {
            let r: Vec<String> = s.split(',').map(|s| s.to_owned()).collect();
            Ok(Some(r))
        }
        None => Ok(None),
    }
}

// impl Default for Mime {
//     fn default() -> Self { limit: None }
// }

// #[derive(Debug, Clone)]
// pub struct App {
//     proxy: Proxy,
//     plex: PlexClient,
// }

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
    #[serde(
        rename = "Directory",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    #[yaserde(rename = "Directory")]
    pub directory: Vec<MetaData>, // only avaiable in XML
    #[serde(rename = "Video", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Video")]
    pub video: Vec<MetaData>, // again only xml, but its the same as directory and metadata
    #[yaserde(attribute, rename = "childCount")]
    #[serde(
        default,
        deserialize_with = "deserialize_option_string_from_number",
        skip_serializing_if = "Option::is_none"
    )]
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
    #[yaserde(rename = "Label", default)]
    #[yaserde(child)]
    pub labels: Vec<Label>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub originally_available_at: Option<String>,
}

pub(crate) fn deserialize_option_string_from_number<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(deserialize_string_from_number(deserializer)?))
}

impl MetaData {
    pub fn children_mut(&mut self) -> &mut Vec<MetaData> {
        if !self.metadata.is_empty() {
            return &mut self.metadata;
        } else if !self.video.is_empty() {
            return &mut self.video;
        } else if !self.directory.is_empty() {
            return &mut self.directory;
        };
        &mut self.metadata
    }

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

    pub fn has_label(&self, name: String) -> bool {
        for label in &self.labels {
            if label.tag == name {
                return true;
            }
        }
        false
        // collection_details.media_container.directory.get(0).unwrap().label.is_some()
    }

    pub fn is_watched(&self) -> bool {
        if self.view_count.is_some() && self.view_count.unwrap_or_default() > 0
        {
            return true;
        }
        if self.viewed_leaf_count.is_some()
            && self.viewed_leaf_count.unwrap_or_default() > 0
        {
            return true;
        }
        false
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

#[derive(
    Debug, Serialize, Deserialize, Clone, YaDeserialize, YaSerialize, Default,
)]
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
    #[serde(
        rename = "librarySectionID",
        skip_serializing_if = "Option::is_none"
    )]
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

#[derive(
    Debug, Serialize, Deserialize, Clone, YaDeserialize, YaSerialize, Default,
)]
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

#[derive(
    Debug, Serialize, Deserialize, Clone, YaDeserialize, YaSerialize, Default,
)]
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

impl MediaContainer {
    pub fn is_hub(&self) -> bool {
        !self.hub.is_empty()
    }

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

    pub fn set_children_mut(&mut self, value: &mut Vec<MetaData>) {
        let len: i32 = value.len().try_into().unwrap();
        if !self.metadata.is_empty() {
            self.metadata = value.to_owned();
        } else if !self.hub.is_empty() {
            self.hub = value.to_owned();
        } else if !self.video.is_empty() {
            self.video = value.to_owned();
        } else if !self.directory.is_empty() {
            self.directory = value.to_owned();
        };
        self.size = Some(len);
    }

    pub fn children_mut(&mut self) -> &mut Vec<MetaData> {
        if !self.metadata.is_empty() {
            return &mut self.metadata;
        } else if !self.hub.is_empty() {
            return &mut self.hub;
        } else if !self.video.is_empty() {
            return &mut self.video;
        } else if !self.directory.is_empty() {
            return &mut self.directory;
        };
        &mut self.metadata
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

/// NOTICE: Cant set yaserde on this? it will complain about a generic
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct MediaContainerWrapper<T> {
    #[serde(rename = "MediaContainer")]
    // #[serde(rename="$value")]
    // #[yaserde(child)]
    pub media_container: T,
    #[serde(skip_serializing, skip_deserializing)]
    // #[yaserde(attribute)]
    pub content_type: ContentType,
}

// impl Default for MediaContainerWrapper {
//     fn default() -> Self { limit: None }
// }

#[async_trait]
pub trait FromResponse<T>: Sized {
    async fn from_response(resp: T) -> Result<Self>;
}

impl MediaContainerWrapper<MediaContainer> {
    pub fn is_hub(&self) -> bool {
        !self.media_container.hub.is_empty()
    }

    pub fn is_section_hub(&self) -> bool {
        self.is_hub() && self.media_container.library_section_id.is_some()
    }
}
