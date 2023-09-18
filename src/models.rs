use figment::{providers::Env, Figment};
use salvo::prelude::*;
use std::f32::consts::E;
use std::fmt;
use std::str::FromStr;
use std::string::ToString;
use tmdb_api::movie::images::MovieImages;
use tmdb_api::prelude::Command;
use tmdb_api::tvshow::search::TVShowSearch;
use tmdb_api::Client;

extern crate mime;
use crate::cache::GLOBAL_CACHE;
use crate::config::*;
use crate::plex_client::PlexClient;
use crate::utils::*;
use anyhow::Result;
use async_trait::async_trait;
use serde_aux::prelude::{
    deserialize_number_from_string, deserialize_string_from_number,
};
// use smartstring::alias::String;
// use hyper::Body;
use itertools::Itertools;
use salvo::http::ReqBody;
use salvo::http::ResBody;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::serde_as;
use std::io::{Read, Write};
use strum_macros::Display as EnumDisplay;
use strum_macros::EnumString;
use tracing::debug;
use xml::writer::XmlEvent;
use yaserde::YaSerialize as YaSerializeTrait;
use yaserde_derive::YaDeserialize;
use yaserde_derive::YaSerialize;

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

#[derive(
    Debug, Clone, PartialEq, Eq, EnumString, EnumDisplay, Serialize, Deserialize,
)]
pub enum Style {
    #[serde(rename = "hero")]
    Hero,
    #[serde(rename = "shelf")]
    Shelf
}

fn default_as_false() -> bool {
    false
}

#[derive(
    Debug, Clone, PartialEq, Eq, EnumString, EnumDisplay, Serialize, Deserialize,
)]
pub enum Platform {
    // #[serde(default)]
    Android,
    #[serde(rename = "iOS")]
    #[strum(serialize = "iOS")]
    Ios,
    Safari,
    Chrome,
    Roku,
    #[serde(other)]
    #[strum(serialize = "Generic")]
    Generic,
}

impl Default for Platform {
    fn default() -> Self {
        Platform::Generic
    }
}

#[derive(Serialize, Deserialize, Debug, Extractible, Default, Clone)]
#[salvo(extract(
    default_source(from = "query"),
    default_source(from = "header"),
    rename_all = "camelCase"
))]
pub struct Resolution {
    pub height: i64,
    pub width: i64,
}

#[derive(Serialize, Deserialize, Debug, Extractible, Default, Clone)]
#[salvo(extract(
    default_source(from = "query"),
    default_source(from = "header"),
    rename_all = "camelCase"
))]
pub struct PlexContext {
    #[serde(default, deserialize_with = "deserialize_comma_seperated_string")]
    #[salvo(extract(rename = "contentDirectoryID"))]
    pub content_directory_id: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_comma_seperated_string")]
    #[salvo(extract(rename = "pinnedContentDirectoryID"))]
    pub pinned_content_directory_id: Option<Vec<String>>,
    #[serde(default="default_platform")]
    #[salvo(extract(rename = "X-Plex-Platform"))]
    pub platform: Platform,
    #[serde(default, deserialize_with = "deserialize_screen_resolution")]
    #[salvo(extract(rename = "X-Plex-Device-Screen-Resolution"))]
    pub screen_resolution: Vec<Resolution>,
    #[salvo(extract(rename = "X-Plex-Device-Screen-Resolution"))]
    pub screen_resolution_original: Option<String>,
    #[salvo(extract(rename = "x-plex-client-capabilities"))]
    pub client_capabilities: Option<String>, 
    #[salvo(extract(rename = "X-Plex-Product"))]
    pub product: Option<String>,
    #[salvo(extract(rename = "X-Plex-Version"))]
    pub version: Option<String>,
    pub count: Option<i32>,
    #[salvo(extract(rename = "X-Plex-Client-Identifier"))]
    pub client_identifier: Option<String>,
    #[salvo(extract(rename = "X-Plex-Session-Id"))]
    pub session_id: Option<String>,
    #[salvo(extract(rename = "X-Plex-Session-Identifier"))]
    pub session_identifier: Option<String>,
    #[salvo(extract(rename = "X-Plex-Playback-Session-Id"))]
    pub playback_session_id: Option<String>,
    #[salvo(extract(rename = "X-Plex-Playback-Id"))]
    pub playback_id: Option<String>,
    #[salvo(extract(rename = "X-Plex-Token"))]
    pub token: Option<String>,
    #[salvo(extract(rename = "X-Plex-Platform-Version"))]
    pub platform_version: Option<String>,
    #[salvo(extract(rename = "X-Plex-Features"))]
    pub features: Option<String>,
    #[salvo(extract(rename = "X-Plex-Model"))]
    pub model: Option<String>,
    #[salvo(extract(rename = "X-Plex-Device"))]
    pub device: Option<String>,
    #[salvo(extract(rename = "X-Plex-Device-Name"))]
    pub device_name: Option<String>,
    #[salvo(extract(rename = "X-Plex-Drm"))]
    pub drm: Option<String>,
    #[salvo(extract(rename = "X-Plex-Text-Format"))]
    pub text_format: Option<String>,
    #[salvo(extract(rename = "X-Plex-Provider-Version"))]
    pub provider_version: Option<String>,
    #[salvo(extract(rename = "X-Plex-Container-Size"))]
    pub container_size: Option<i32>,
    #[salvo(extract(rename = "X-Plex-Container-Start"))]
    pub container_start: Option<i32>,
    #[salvo(extract(rename = "x-plex-http-pipeline"))]
    pub http_pipeline: Option<String>,
    #[serde(default = "default_as_false", deserialize_with = "bool_from_int")]
    #[salvo(extract(rename = "includeCollections"))]
    pub include_collections: bool,
    #[serde(default = "default_as_false", deserialize_with = "bool_from_int")]
    #[salvo(extract(rename = "includeAdvanced"))]
    pub include_advanced: bool,
    #[salvo(extract(rename = "X-Forwarded-For", alias = "X-Real-Ip"))]
    pub forwarded_for: Option<String>,
    #[salvo(extract(rename = "X-Forwarded-Proto"))]
    pub forwarded_proto: Option<String>,
    #[salvo(extract(rename = "x-forwarded-host"))]
    pub forwarded_host: Option<String>,
    #[salvo(extract(rename = "X-Forwarded-Port"))]
    pub forwarded_port: Option<String>,
    #[serde(default = "default_as_false", deserialize_with = "bool_from_int")]
    #[salvo(extract(rename = "excludeAllLeaves"))]
    pub exclude_all_leaves: bool,
    // photo transcode
    pub size: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub quality: Option<i32>,
    pub url: Option<String>,

    // this our own fields
    // pub style: Option<Style>,
}

fn default_platform() -> Platform {
    Platform::Generic
}

fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}

pub fn deserialize_comma_seperated_number<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<i32>>, D::Error>
where
    D: Deserializer<'de>,
{
    match Deserialize::deserialize(deserializer)? {
        Some::<String>(s) => {
            if s.is_empty() {
                return Ok(None);
            }
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

// pub fn deserialize_comma_seperated_string<'de, D>(
//     deserializer: D,
// ) -> Result<Option<i64>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     match Deserialize::deserialize(deserializer)? {
//         Some::<String>(s) => {
//             let r: Vec<String> = s.split(',').map(|s| s.to_owned()).collect();
//             Ok(Some(r))
//         }
//         None => Ok(None),
//     }
// }

pub fn deserialize_screen_resolution<'de, D>(
    deserializer: D,
) -> Result<Vec<Resolution>, D::Error>
where
    D: Deserializer<'de>,
{
    match Deserialize::deserialize(deserializer)? {
        Some::<String>(s) => {
            let cleaned_string: String = s
                .chars()
                .filter(|c| c.is_numeric() || *c == 'x' || *c == ',')
                .collect();
            let r: Vec<Resolution> = cleaned_string
                .split(',')
                .map(|s| {
                    let k: Vec<i64> =
                        s.split("x").map(|s| s.parse().unwrap()).collect();
                    Resolution {
                        width: k[0],
                        height: k[1],
                    }
                })
                .collect();
            Ok(r)
        }
        None => Ok(vec![]),
    }
}

// fn skip_on_error<'de, D>(deserializer: D) -> Result<Option<>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     // Ignore the data in the input.
//     match Deserialize::deserialize(deserializer) {
//         Ok(v) => v,
//         Err(e) => Ok(None),
//     }
// }

// pub fn deserialize_optional_datetime<'de, D>(d: D) -> Result<Option<DateTime<Utc>>, D::Error>
// where
//     D: de::Deserializer<'de>,
// {
//     d.deserialize_option(OptionalDateTimeFromCustomFormatVisitor)
// }

// struct OptionalIntFromStr;

pub fn optional_int_from_str<'de, D: Deserializer<'de>>(
    de: D,
) -> Result<Option<i64>, D::Error> {
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Option<i64>;

        fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt.write_str("integer or string")
        }

        // fn visit_i64<E>(self, val: i64) -> Result<Self::Value, E>
        // where
        //     E: serde::de::Error,
        // {
        //     match NonZeroU32::new(val as i32) {
        //         Some(val) => Ok(MyType(val)),
        //         None => Err(E::custom("invalid integer value")),
        //     }
        // }

        fn visit_str<E>(self, val: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            dbg!("Matcging");
            match val.parse::<i64>() {
                Ok(val) => Ok(Some(val)),
                Err(_) => Err(E::custom("failed to parse integer")),
            }
        }
    }

    de.deserialize_option(Visitor)
}

// pub fn int_from_str<'de, D>(
//     deserializer: D,
// ) -> Result<Option<i64>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     match Deserialize::deserialize(deserializer)? {
//         Some::<i64>(s) => {
//             Ok(Some(s))
//         },
//         None => Ok(None),
//     }
// }

// pub fn int_from_str<'de, D: Deserializer<'de>>(de: D) -> Result<Option<i64>, D::Error> {
//     struct Visitor;

//     impl<'de> serde::de::Visitor<'de> for Visitor {
//         type Value = Option<i64>;

//         fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//             f.write_str("a boolean")
//         }

//         fn visit_str<E: serde::de::Error>(self, val: &str) -> Result<Option<i64>, E> {
//            dbg!("yess");
//            Ok(Some(val.parse::<i64>().unwrap()))
//             // match val {
//             //     v if uncased::eq(v, "true") => Ok(true),
//             //     v if uncased::eq(v, "false") => Ok(false),
//             //     s => Err(E::invalid_value(Unexpected::Str(s), &"true or false"))
//             // }
//         }

//         // fn visit_u64<E: de::Error>(self, n: u64) -> Result<bool, E> {
//         //     match n {
//         //         0 | 1 => Ok(n != 0),
//         //         n => Err(E::invalid_value(Unexpected::Unsigned(n), &"0 or 1"))
//         //     }
//         // }

//         // fn visit_i64<E: de::Error>(self, n: i64) -> Result<bool, E> {
//         //     match n {
//         //         0 | 1 => Ok(n != 0),
//         //         n => Err(E::invalid_value(Unexpected::Signed(n), &"0 or 1"))
//         //     }
//         // }
//     }

//     de.deserialize_any(Visitor)
// }

/// For some fucking reason. Android for mobile (and only that) chokes on boolean (true/false) in xml. It wants 0/1
#[derive(Debug, Clone, PartialEq, Eq, Default, PartialOrd, YaDeserialize)]
pub struct SpecialBool {
    inner: bool,
}

impl SpecialBool {
    pub fn new(inner: bool) -> Self {
        Self { inner }
    }
}

impl fmt::Display for SpecialBool {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl YaSerializeTrait for SpecialBool {
    fn serialize<W: Write>(
        &self,
        writer: &mut yaserde::ser::Serializer<W>,
    ) -> Result<(), String> {
        let content = format!("{}", self.inner as i64);
        let event = XmlEvent::characters(&content);
        let _ret = writer.write(event);
        Ok(())
    }

    fn serialize_attributes(
        &self,
        attributes: Vec<xml::attribute::OwnedAttribute>,
        namespace: xml::namespace::Namespace,
    ) -> Result<
        (
            Vec<xml::attribute::OwnedAttribute>,
            xml::namespace::Namespace,
        ),
        String,
    > {
        Ok((attributes, namespace))
    }
}

impl Serialize for SpecialBool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // If you implement `Deref`, then you don't need to add `.0`
        let s = self.inner;
        serializer.serialize_bool(s)
    }
}

impl<'de> Deserialize<'de> for SpecialBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s =
            serde_aux::prelude::deserialize_bool_from_anything(deserializer)
                .unwrap();
        Ok(SpecialBool::new(s))
    }

    // fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    // where
    //     V: Visitor<'de>;
}

// impl FromStr for SpecialBool {
//     type Err = ParseBoolError
//     fn from_str(s: &str) -> Result<bool, ParseBoolError> {

//     }
// }

// pub fn deserialize_special_bool<'de, D>(
//     deserializer: D,
// ) -> Result<Option<SpecialBool>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     match Deserialize::deserialize(deserializer)? {
//         Some::<bool>(s) => {
//             Ok(Some(SpecialBool::new(s)))
//         }
//         None => Ok(None),
//     }
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
pub struct Guid {
    #[yaserde(attribute)]
    id: String,
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
pub struct Tag {
    #[yaserde(attribute)]
    tag: String,
}


#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    // Eq,
    YaDeserialize,
    YaSerialize,
    Default,
    PartialOrd,
)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct Media {
    #[yaserde(attribute)]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: i64,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
    #[yaserde(attribute, rename = "bitrate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<i64>,
    //#[yaserde(attribute, rename = "aspectRatio")]
    //#[serde(skip_serializing_if = "Option::is_none")]
    //pub aspect_ratio: Option<f64>,
    #[yaserde(attribute, rename = "audioChannels")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_channels: Option<i64>,
    #[yaserde(attribute, rename = "audioCodec")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_codec: Option<String>,
    #[yaserde(attribute, rename = "videoCodec")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_codec: Option<String>,
    #[yaserde(attribute, rename = "videoResolution")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_resolution: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i64>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i64>,
    #[yaserde(attribute, rename = "partCount")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_count: Option<i32>,
    #[yaserde(attribute, rename = "channelArt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_art: Option<String>,
    #[yaserde(attribute, rename = "videoProfile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_profile: Option<String>,
    #[yaserde(attribute, rename = "videoFrameRate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_frame_rate: Option<String>,
    #[yaserde(attribute, rename = "container")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[yaserde(attribute, rename = "optimizedForStreaming")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimized_for_streaming: Option<SpecialBool>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    #[yaserde(rename = "Part", child)]
    #[serde(skip_serializing_if = "Vec::is_empty", default, rename = "Part")]
    pub parts: Vec<MediaPart>,
}

impl fmt::Display for Media {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} - {} - {}",
            self.video_resolution.clone().unwrap_or_default(),
            self.video_codec.clone().unwrap_or_default(),
            self.audio_codec.clone().unwrap_or_default()
        )
    }
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    // Eq,
    YaDeserialize,
    YaSerialize,
    Default,
    PartialOrd,
)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct MediaPart {
    #[yaserde(attribute)]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: i64,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[yaserde(attribute, rename = "container")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    #[yaserde(attribute, rename = "videoProfile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_profile: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[yaserde(attribute, rename = "optimizedForStreaming")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimized_for_streaming: Option<SpecialBool>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    #[yaserde(rename = "Stream", child)]
    #[serde(skip_serializing_if = "Vec::is_empty", default, rename = "Stream")]
    pub streams: Vec<Stream>,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    // Eq,
    YaDeserialize,
    YaSerialize,
    Default,
    PartialOrd,
)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    #[yaserde(attribute)]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: i64,
    #[yaserde(attribute, rename = "streamType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_type: Option<i64>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codec: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<i64>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[yaserde(attribute, rename = "languageTag")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_tag: Option<String>,
    #[yaserde(attribute, rename = "languageCode")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
    #[yaserde(attribute, rename = "DOVIBLCompatID")]
    #[serde(
        rename = "DOVIBLCompatID",
        skip_serializing_if = "Option::is_none"
    )]
    pub doviblcompat_id: Option<i64>,
    #[yaserde(attribute, rename = "DOVIBLPresent")]
    #[serde(rename = "DOVIBLPresent", skip_serializing_if = "Option::is_none")]
    pub doviblpresent: Option<bool>,
    #[yaserde(attribute, rename = "DOVIELPresent")]
    #[serde(rename = "DOVIELPresent")]
    pub dovielpresent: Option<bool>,
    #[yaserde(attribute, rename = "DOVILevel")]
    #[serde(rename = "DOVILevel", skip_serializing_if = "Option::is_none")]
    pub dovilevel: Option<i64>,
    #[yaserde(attribute, rename = "DOVIPresent")]
    #[serde(rename = "DOVIPresent", skip_serializing_if = "Option::is_none")]
    pub dovipresent: Option<bool>,
    #[yaserde(attribute, rename = "DOVIProfile")]
    #[serde(rename = "DOVIProfile", skip_serializing_if = "Option::is_none")]
    pub doviprofile: Option<i64>,
    #[yaserde(attribute, rename = "DOVIRPUPresent")]
    #[serde(
        rename = "DOVIRPUPresent",
        skip_serializing_if = "Option::is_none"
    )]
    pub dovirpupresent: Option<bool>,
    #[yaserde(attribute, rename = "DOVIVersion")]
    #[serde(rename = "DOVIVersion", skip_serializing_if = "Option::is_none")]
    pub doviversion: Option<String>,
    #[yaserde(attribute, rename = "bitDepth")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_depth: Option<i64>,
    #[yaserde(attribute, rename = "chromaLocation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chroma_location: Option<String>,
    #[yaserde(attribute, rename = "chromaSubsampling")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chroma_subsampling: Option<String>,
    #[yaserde(attribute, rename = "codeHeight")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coded_height: Option<i64>,
    #[yaserde(attribute, rename = "codeWidth")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coded_width: Option<i64>,
    #[yaserde(attribute, rename = "colorPrimaries")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_primaries: Option<String>,
    #[yaserde(attribute, rename = "colorRange")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_range: Option<String>,
    #[yaserde(attribute, rename = "colorSpace")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_space: Option<String>,
    #[yaserde(attribute, rename = "colorTrc")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_trc: Option<String>,
    #[yaserde(attribute, rename = "frameRate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_rate: Option<f64>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i64>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<i64>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original: Option<bool>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[yaserde(attribute, rename = "refFrames")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_frames: Option<i64>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i64>,
    #[yaserde(attribute, rename = "displayTitle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_title: Option<String>,
    #[yaserde(attribute, rename = "extendedDisplaytitle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_display_title: Option<String>,
    #[yaserde(attribute, rename = "hasScalingMatrix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_scaling_matrix: Option<bool>,
    #[yaserde(attribute, rename = "scanType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_type: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<i64>,
    #[yaserde(attribute, rename = "audioChannelLayout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_channel_layout: Option<String>,
    #[yaserde(attribute, rename = "samplingRate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling_rate: Option<i64>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forced: Option<bool>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[yaserde(attribute, rename = "hearingImpaired")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hearing_impaired: Option<bool>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
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
pub struct Image {
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
    #[serde(rename = "type")]
    #[yaserde(attribute, rename = "type")]
    pub r#type: String,
    #[yaserde(attribute)]
    pub url: String,
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
    #[serde(deserialize_with = "deserialize_number_from_string")]
    id: i64,
    #[yaserde(attribute)]
    tag: String,
    #[yaserde(attribute)]
    filter: String,
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
pub struct Context {
    #[serde(rename = "Image", default, skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Image", default, child)]
    pub images: Vec<Image>,
}

// #[derive(Debug)]
// struct FailableOption<T: Default>(T);

// impl<'de, T: Default + Deserialize<'de>> Deserialize<'de> for OrDefault<T> {
//     fn or_default<'de, D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
//         T::deserialize(d).or_else(|_| Ok(T::default())).map(OrDefault)
//     }
// }

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guid: Option<String>,
    #[yaserde(attribute, rename = "primaryGuid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_guid: Option<String>,
    #[yaserde(attribute)]
    // #[yaserde(skip_serializing = true)]
    // #[serde(skip_serializing)]
    pub title: String,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
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
    #[yaserde(attribute, rename = "addedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added_at: Option<i64>,
    #[yaserde(attribute, rename = "updatedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    #[yaserde(attribute, rename = "lastViewedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_viewed_at: Option<i64>,
    #[yaserde(attribute, rename = "includedAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_mode: Option<i32>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub art: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i32>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub studio: Option<String>,
    #[yaserde(attribute, rename = "contentRating")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_rating: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<f64>,
    #[yaserde(attribute, rename = "audienceRating")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience_rating: Option<f64>,
    #[yaserde(attribute, rename = "viewOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_offset: Option<i64>,
    //#[yaserde(attribute, rename = "skipCount")]
    //#[serde(skip_serializing_if = "Option::is_none")]
    //pub skip_count: Option<i64>,
    #[yaserde(attribute, rename = "primaryExtraKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_extra_key: Option<String>,
    #[yaserde(attribute, rename = "chapterSource")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_source: Option<String>,
    #[yaserde(attribute, rename = "ratingImage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating_image: Option<String>,
    #[yaserde(attribute, rename = "audienceRatingImage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audiance_rating_image: Option<String>,
    #[yaserde(attribute, rename = "parentYear")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_year: Option<i32>,
    #[yaserde(attribute, rename = "parentIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_index: Option<u32>,
    #[yaserde(attribute, rename = "parentGuid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_guid: Option<String>,
    #[yaserde(attribute, rename = "parentStudio")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_studio: Option<String>,
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
    #[yaserde(rename = "parentArt")]
    pub parent_art: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "parentThumb")]
    pub parent_thumb: Option<String>,
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
    #[yaserde(rename = "librarySectionID")]
    #[yaserde(attribute)]
    #[serde(
        default,
        rename = "librarySectionID",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub library_section_id: Option<i64>,
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
    pub promoted: Option<SpecialBool>,
    #[yaserde(attribute, rename = "skipDetails")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_details: Option<SpecialBool>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<bool>,
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
    // #[serde(skip_serializing_if = "Option::is_none", deserialize_with="deserialize_special_bool")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more: Option<SpecialBool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(attribute)]
    pub style: Option<String>,
    // #[yaserde(skip)]
    #[yaserde(attribute, rename = "Meta")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "Meta")]
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
    #[yaserde(attribute, rename = "originallyAvailableAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub originally_available_at: Option<String>,
    #[serde(rename = "Media", default, skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Media", default, child)]
    pub media: Vec<Media>,
    #[serde(rename = "Guid", default, skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Guid", default, child)]
    pub guids: Vec<Guid>,
    #[yaserde(attribute, rename = "userState")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_state: Option<SpecialBool>,
    #[serde(rename = "Image", default, skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Image", default, child)]
    pub images: Vec<Image>,
    #[serde(rename = "Context", skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "Context", child)]
    pub context_images: Option<Context>,
    #[yaserde(attribute)]
    #[yaserde(rename = "extraType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_type: Option<i32>, // actually a bool but plex does 0 and 1
    #[yaserde(attribute)]
    #[yaserde(rename = "playQueueItemID")]
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "playQueueItemID"
    )]
    pub play_queue_item_id: Option<i64>,
    #[serde(rename = "Collection", default, skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Collection", default, child)]
    pub collections: Vec<Tag>,
    #[serde(rename = "Country", default, skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Country", default, child)]
    pub countries: Vec<Tag>,
    #[serde(rename = "Director", default, skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Director", default, child)]
    pub directors: Vec<Tag>,
    #[serde(rename = "Genre", default, skip_serializing_if = "Vec::is_empty")]
    #[yaserde(rename = "Genre", default, child)]
    pub genres: Vec<Tag>,
}

pub(crate) fn deserialize_option_string_from_number<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(deserialize_string_from_number(deserializer)?))
}

pub(crate) fn deserialize_option_number_from_string<'de, D>(
    deserializer: D,
) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize::deserialize(deserializer)
    // if s.parse::<f64>().is_ok() {
    //    return s.parse::<f64>()
    // }

    match serde_aux::prelude::deserialize_option_number_from_string::<i64, D>(
        deserializer,
    ) {
        Ok(r) => Ok(r),
        Err(_) => Ok(None),
    }
    // let b = deserialize_number_from_string::<i64, D>(deserializer)?;
    // // dbg!(&b);
    // Ok(Some(b))
}

impl MetaData {
    // TODO: move to plexclient
    pub async fn get_hero_art(
        &self,
        plex_client: PlexClient,
    ) -> Option<String> {
        self.guid.as_ref()?;
        let mut guid = self.guid.clone().unwrap();
        if guid.starts_with("local://") {
            tracing::debug!(
                "Skipping loading remote metadata for local item: {}",
                guid,
            );
            return None;
        }
        
        if guid.starts_with("plex://episode") && self.parent_guid.is_some() {    
             guid = self.parent_guid.clone().unwrap();
        //     dbg!(&guid);
        }

        let cache_key = format!("{}:cover_art", guid);

        let cached_result: Option<Option<String>> =
            GLOBAL_CACHE.get(cache_key.as_str()).await;

        if cached_result.is_some() {
            return cached_result.unwrap();
        }
        let guid = self
            .guid
            .clone()
            .unwrap()
            .replace("plex://show/", "")
            .replace("plex://movie/", "")
            .replace("plex://season/", "")
            .replace("plex://episode/", "");

        let mut container: MediaContainerWrapper<MediaContainer> =
            match plex_client.get_provider_data(guid).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::warn!(
                        "Problem loading prodiver metadata for: {} Error: {}",
                        self.guid.clone().unwrap(),
                        e
                    );
                    MediaContainerWrapper::default()
                }
            };
        // let mut container = plex_client.get_provider_data(guid).await.unwrap();
        let metadata = container.media_container.children_mut().get(0);
        let mut image: Option<String> = None;
        if metadata.is_some() {
            for i in &metadata.unwrap().images {
                if i.r#type == "coverArt" {
                    image = Some(i.url.clone());
                    break;
                }
            }
        }

        let _ = GLOBAL_CACHE
            .insert(cache_key, image.clone(), crate::cache::Expiration::Month)
            .await;
        image
    }

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
            if label.tag.to_lowercase() == name.to_lowercase() {
                return true;
            }
        }
        false
    }

    /// if this hub should be hero style
    pub async fn is_hero(&self, plex_client: PlexClient) -> Result<bool> {
        if !self.is_hub() {
            return Ok(false);
        }
        let config: Config = Config::figment().extract().unwrap();
        // dbg!(&config.hero_rows);
        if config.hero_rows.is_some() && self.hub_identifier.is_some() {
            let id = self.hub_identifier.clone().unwrap();
            for row in config.hero_rows.unwrap() {
                if !row.is_empty() && id.contains(&row) {
                    return Ok(true);
                }
            }
        }
        if !self.is_collection_hub() {
            return Ok(false);
        }
        let collection_id = get_collection_id_from_hub(self);
        let mut collection_details = plex_client
            .clone()
            .get_cached(
                plex_client.get_collection(collection_id),
                format!("collection:{}", collection_id),
            )
            .await?;
        // dbg!(collection_details.media_container.library_section_id);
        Ok(collection_details
            .media_container
            .children()
            .get(0)
            .unwrap()
            .has_label("REPLEXHERO".to_string()))
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
#[serde_as]
#[serde(rename_all = "camelCase")]
#[yaserde(root = "MediaContainer")]
pub struct MediaContainer {
    #[yaserde(attribute)]
    //#[serde(deserialize_with = "str_or_i32")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
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
    pub allow_sync: Option<SpecialBool>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[yaserde(attribute, rename = "librarySectionID")]
    #[serde(
        default,
        rename = "librarySectionID",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub library_section_id: Option<i64>,
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
    #[yaserde(attribute)]
    #[yaserde(rename = "playQueueID")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "playQueueID")]
    pub play_queue_id: Option<i64>,
    #[yaserde(attribute)]
    #[yaserde(rename = "playQueueSelectedItemID")]
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "playQueueSelectedItemID"
    )]
    pub play_queue_selected_item_id: Option<i64>,
    #[yaserde(attribute)]
    #[yaserde(rename = "playQueueSelectedItemOffset")]
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "playQueueSelectedItemOffset"
    )]
    pub play_queue_selected_item_offset: Option<i32>,
    #[yaserde(attribute)]
    #[yaserde(rename = "playQueueSelectedMetadataItemID")]
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "playQueueSelectedMetadataItemID"
    )]
    pub play_queue_selected_metadata_item_id: Option<String>,
    #[yaserde(attribute)]
    #[yaserde(rename = "playQueueShuffled")]
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "playQueueShuffled"
    )]
    pub play_queue_shuffled: Option<bool>,
    #[yaserde(attribute)]
    #[yaserde(rename = "playQueueSourceURI")]
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "playQueueSourceURI"
    )]
    pub play_queue_source_uri: Option<String>,
    #[yaserde(attribute)]
    #[yaserde(rename = "playQueueTotalCount")]
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "playQueueTotalCount"
    )]
    pub play_queue_total_count: Option<i32>,
    #[yaserde(attribute)]
    #[yaserde(rename = "playQueueVersion")]
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "playQueueVersion"
    )]
    pub play_queue_version: Option<i32>,
    #[yaserde(attribute)]
    #[yaserde(rename = "mediaTagPrefix")]
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "mediaTagPrefix"
    )]
    pub media_tag_prefix: Option<String>,
    #[yaserde(attribute)]
    #[yaserde(rename = "mediaTagVersion")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "mediaTagVersion",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub media_tag_version: Option<i64>,
    #[yaserde(attribute)]
    #[yaserde(rename = "directPlayDecisionCode")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "directPlayDecisionCode",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub direct_play_decision_code: Option<i64>,
    #[yaserde(attribute, rename = "directPlayDecisionText")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "directPlayDecisionText",
    )]
    pub direct_play_decision_text: Option<String>,
    #[yaserde(attribute, rename = "generalDecisionCode")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "generalDecisionCode",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub general_decision_code: Option<i64>,
    #[yaserde(attribute, rename = "generalDecisionText")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "generalDecisionText",
    )]
    pub general_decision_text: Option<String>,
    #[yaserde(attribute, rename = "resourceSession")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "resourceSession",
    )]
    pub resource_session: Option<String>,
    #[yaserde(attribute, rename = "transcodeDecisionCode")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "transcodeDecisionCode",
        deserialize_with = "deserialize_option_number_from_string"
    )]
    pub transcode_decision_code: Option<i64>,
    #[yaserde(attribute, rename = "transcodeDecisionText")]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "transcodeDecisionText",
    )]
    pub transcode_decision_text: Option<String>,

    #[yaserde(attribute, rename = "Meta")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "Meta")]
    pub meta: Option<Meta>,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<String>,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, YaDeserialize, YaSerialize, Default,
)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct MetaType {
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "type")]
    pub r#type: Option<String>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, YaDeserialize, YaSerialize, Default,
)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct DisplayImage {
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "type")]
    pub r#type: Option<String>,
    // #[yaserde(attribute)]
    #[yaserde(attribute, rename = "imageType")]
    #[serde(skip_serializing_if = "Option::is_none", rename = "imageType")]
    pub image_type: Option<String>,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, YaDeserialize, YaSerialize, Default,
)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    #[serde(rename = "DisplayFields", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub display_fields: Vec<DisplayField>,
    #[serde(rename = "DisplayImage", default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub display_images: Vec<DisplayImage>,
    #[yaserde(attribute)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[yaserde(rename = "type")]
    pub r#type: Option<MetaType>,
    // #[yaserde(attribute)]
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub style: Option<String>,
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
        let len: i64 = value.len().try_into().unwrap();
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
        let len: i64 = value.len().try_into().unwrap();
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
