use serde::{Deserialize, Serialize};
// use parse_display::{Display, FromStr};
// use yaserde_derive::{YaDeserialize, YaSerialize};


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, YaDeserialize, YaSerialize, Default)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, YaDeserialize, YaSerialize, Default)]
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
    metadata: Vec<MetaData>,
    #[serde(rename = "Directory", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Directory")]
    directory: Vec<MetaData>,   // only avaiable in XML
    #[serde(rename = "Video", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Video")]
    video: Vec<MetaData>,   // again only xml, but its the same as directory and metadata
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, YaDeserialize, YaSerialize, Default)]
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
    pub library_section_id: Option<u32>,
    #[yaserde(attribute)]
    #[yaserde(rename = "librarySectionTitle")]
    pub library_section_title: Option<String>,
    #[serde(rename = "librarySectionUUID")]
    #[yaserde(rename = "librarySectionUUID")]
    #[yaserde(attribute)]
    pub library_section_uuid: Option<String>,
    #[serde(rename = "Hub", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Hub")]
    pub hub: Vec<Hub>,
    #[serde(rename = "Metadata", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub metadata: Vec<MetaData>,
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
}