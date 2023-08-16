extern crate tracing;
use anyhow::Result;
use bytes::Bytes;
extern crate mime;
// use futures_util::StreamExt;
use mime::Mime;
use salvo::Error;
use serde::{Deserialize, Serialize};
use strum_macros::Display as EnumDisplay;
use strum_macros::EnumString;
// use http_body::{Limited, Full};
use http_body_util::BodyExt;
use tracing::error;
use yaserde::ser::to_string as to_xml_str;
// use salvo_core::http::response::Response as SalvoResponse;
use salvo::http::HeaderValue;

use salvo::http::HeaderMap;
use salvo::{
    http::response::Response as SalvoResponse, test::ResponseExt, Extractible,
    Request as SalvoRequest,
};

use crate::models::*;

pub fn get_collection_id_from_child_path(path: String) -> i32 {
    let mut path = path.replace("/library/collections/", "");
    path = path.replace("/children", "");
    path.parse().unwrap()
}

pub fn get_collection_id_from_hub(hub: &MetaData) -> i32 {
    hub.hub_identifier
        .clone()
        .unwrap()
        .split('.')
        .last()
        .unwrap()
        .parse()
        .unwrap()
}

pub fn add_query_param_salvo(
    req: &mut SalvoRequest,
    param: String,
    value: String,
) {
    let mut uri = pathetic::Uri::default()
        .with_path(req.uri_mut().path())
        .with_query(req.uri_mut().query());
    let mut query: Vec<(String, String)> = uri // remove existing values
        .query_pairs()
        .filter(|(name, _)| name.to_string() != param.to_string())
        .map(|(name, value)| (name.into_owned(), value.into_owned()))
        .collect();
    query.push((param.to_owned(), value.to_owned()));
    uri.query_pairs_mut().clear().extend_pairs(query);
    *req.uri_mut() = hyper::Uri::try_from(uri.as_str()).unwrap();
    // req
}

#[derive(
    Debug, Clone, PartialEq, Eq, EnumString, EnumDisplay, Serialize, Deserialize,
)]
pub enum ContentType {
    #[strum(serialize = "application/json", serialize = "text/json")]
    Json,
    #[strum(
        serialize = "text/xml;charset=utf-8",
        serialize = "application/xml"
    )]
    Xml,
}

impl Default for ContentType {
    fn default() -> Self {
        ContentType::Xml
    }
}

pub fn get_content_type_from_headers(
    headers: &HeaderMap<HeaderValue>,
) -> ContentType {
    let default_header_value =
        HeaderValue::from_static("text/xml;charset=utf-8");
    let accept_header = headers.get("accept");
    let content_type_header = headers.get("content-type");

    let content_type = if content_type_header.is_some() {
        content_type_header.unwrap()
    } else if accept_header.is_some() {
        accept_header.unwrap()
    } else {
        &default_header_value
    }
    .to_str()
    .unwrap();

    match content_type {
        x if x.contains("application/json") => ContentType::Json,
        x if x.contains("text/xml") => ContentType::Xml,
        _ => ContentType::Xml,
    }
}

// pub fn get_content_type(req: Request<Body>) -> ContentType {
//     let (parts, _body) = req.into_parts();
//     get_content_type_from_headers(&parts.headers)
// }

pub fn mime_to_content_type(mime: Mime) -> ContentType {
    match (mime.type_(), mime.subtype()) {
        (mime::JSON, _) => ContentType::Json,
        (mime::XML, _) => ContentType::Xml,
        _ => ContentType::Xml,
    }
}

pub fn from_string(
    string: String,
    content_type: mime::Mime,
) -> Result<MediaContainerWrapper<MediaContainer>> {
    // dbg!(&string);
    // dbg!(&content_type.subtype());
    let result: MediaContainerWrapper<MediaContainer> =
        match (content_type.type_(), content_type.subtype()) {
            (_, mime::JSON) => {
                let mut c: MediaContainerWrapper<MediaContainer> =
                    serde_json::from_str(&string).unwrap();
                c.content_type = ContentType::Json;
                c
            }
            _ => MediaContainerWrapper {
                // default to xml
                // media_container: from_xml_str(&body_string).unwrap(),
                media_container: yaserde::de::from_str(&string).unwrap(),
                content_type: ContentType::Xml,
            },
            // _ => "attachment",
        };
    Ok(result)
}

pub async fn from_reqwest_response(
    res: reqwest::Response,
) -> Result<MediaContainerWrapper<MediaContainer>, Error> {
    let bytes = res.bytes().await.unwrap();
    // serde_json::from_reader(&*bytes).map_err(Error::other)
    serde_json::from_reader(&*bytes).map_err(Error::other)
}

pub async fn from_hyper_response(
    res: HyperResponse,
) -> Result<MediaContainerWrapper<MediaContainer>, Error> {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_reader(&*bytes).map_err(Error::other)
}

pub async fn from_salvo_response(
    mut res: SalvoResponse,
) -> Result<MediaContainerWrapper<MediaContainer>, Error> {
    let bytes = res.take_bytes(None).await.unwrap();
    serde_json::from_reader(&*bytes).map_err(Error::other)
}

// pub fn from_bytes(
//     bytes: bytes::Bytes,
//     content_type: ContentType,
// ) -> Result<MediaContainerWrapper<MediaContainer>, Error> {    
//     let result: MediaContainerWrapper<MediaContainer> = match content_type {
//         ContentType::Json => {
//             let mut c: MediaContainerWrapper<MediaContainer> =
//                 serde_json::from_reader(&*bytes).expect("Expected proper json");
//             c.content_type = ContentType::Json;
//             c
//         }
//         ContentType::Xml => MediaContainerWrapper {
//             media_container: yaserde::de::from_reader(&*bytes).unwrap(),
//             content_type: ContentType::Xml,
//         },
//     };
//     Ok(result)
// }

// Nice example of extracting response by content type: https://github.com/salvo-rs/salvo/blob/7122c3c009d7b94e7ecf155fb096f11884a8c01b/crates/core/src/test/response.rs#L47
// TODO: use body not string
// pub async fn from_response(
//     mut res: SalvoResponse,
// ) -> Result<MediaContainerWrapper<MediaContainer>> {
//     // let content_type = get_content_type_from_headers(res.headers_mut());
//     let content_type = res.content_type().unwrap();
//     // let bytes = res.take_bytes(res.content_type().as_ref()).await.unwrap();
//     let string = res.take_string().await.unwrap();
//     // dbg!(&res);

//     // let result = match from_bytes(bytes, &content_type) {
//     let result = match from_string(string, content_type) {
//         Ok(result) => result,
//         Err(error) => {
//             error!("Problem deserializing: {:?}", error);
//             let container: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper::default();
//             container // TOOD: Handle this higher up
//         }
//     };
//     Ok(result)
// }

pub async fn to_string(
    container: MediaContainerWrapper<MediaContainer>,
    content_type: &ContentType,
) -> Result<String> {
    match content_type {
        ContentType::Json => Ok(serde_json::to_string(&container).unwrap()),
        // ContentType::Xml => Ok("".to_owned()),
        ContentType::Xml => Ok(to_xml_str(&container.media_container).unwrap()),
    }
}

// TODO: Merge hub keys when mixed
pub fn merge_children_keys(
    mut key_left: String,
    mut key_right: String,
) -> String {
    key_left = key_left.replace("/hubs/library/collections/", "");
    key_left = key_left.replace("/library/collections/", "");
    key_left = key_left.replace("/children", "");
    key_right = key_right.replace("/hubs/library/collections/", "");
    key_right = key_right.replace("/library/collections/", "");
    key_right = key_right.replace("/children", "");

    format!(
        "/replex/library/collections/{},{}/children",
        key_left,
        key_right // order is important. As thhis order is used to generated the library collections
    )
}
