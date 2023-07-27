extern crate tracing;
use anyhow::Result;
use bytes::{Bytes};
extern crate mime;
use mime::Mime;
use serde::{Deserialize, Serialize};
use strum_macros::Display as EnumDisplay;
use strum_macros::EnumString;

use tracing::debug;
use tracing::error;
use yaserde::ser::to_string as to_xml_str;
// use salvo_core::http::response::Response as SalvoResponse;
use salvo::http::HeaderValue;

use salvo::http::HeaderMap;
use salvo::{
    http::response::Response as SalvoResponse, test::ResponseExt,
    Extractible, Request as SalvoRequest,
};



use crate::models::*;

// pub fn remove_param(mut req: Request<Body>, param: &str) -> Request<Body> {
//     let mut uri = pathetic::Uri::default()
//         .with_path(req.uri_mut().path())
//         .with_query(req.uri_mut().query());
//     let query: Vec<(String, String)> = uri
//         .query_pairs()
//         .filter(|(name, _)| name != param)
//         .map(|(name, value)| (name.into_owned(), value.into_owned()))
//         .collect();
//     uri.query_pairs_mut().clear().extend_pairs(query);
//     *req.uri_mut() = Uri::try_from(uri.as_str()).unwrap();
//     req
// }

// pub fn add_query_param(mut req: Request<Body>, param: &str, value: &str) -> Request<Body> {
//     let mut uri = pathetic::Uri::default()
//         .with_path(req.uri_mut().path())
//         .with_query(req.uri_mut().query());
//     let mut query: Vec<(String, String)> = uri // remove existing values
//         .query_pairs()
//         .filter(|(name, _)| name != param)
//         .map(|(name, value)| (name.into_owned(), value.into_owned()))
//         .collect();
//     query.push((param.to_owned(), value.to_owned()));
//     uri.query_pairs_mut().clear().extend_pairs(query);
//     *req.uri_mut() = Uri::try_from(uri.as_str()).unwrap();
//     req
// }

pub fn add_query_param_salvo(req: &mut SalvoRequest, param: String, value: String) {
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
    Debug,
    Clone,
    PartialEq,
    Eq,
    EnumString,
    EnumDisplay,
    YaDeserialize,
    YaSerialize,
    Serialize,
    Deserialize,
)]
pub enum ContentType {
    #[strum(serialize = "application/json", serialize = "text/json")]
    Json,
    #[strum(serialize = "text/xml;charset=utf-8", serialize = "application/xml")]
    Xml,
}

impl Default for ContentType {
    fn default() -> Self {
        ContentType::Xml
    }
}

pub fn get_content_type_from_headers(headers: &HeaderMap<HeaderValue>) -> ContentType {
    let default_header_value = HeaderValue::from_static("text/xml;charset=utf-8");
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

// TODO: Make this traits of the Hub struct
pub async fn body_to_string(body: hyper::Body) -> Result<String> {
    // dbg!(&body.boxed());
    // dbg!(&body);
    let body_bytes = hyper::body::to_bytes(body).await?;
    // let body_bytes = to_bytes(body).await.unwrap();

    let string = String::from_utf8(body_bytes.to_vec())?;
    // dbg!(&string);
    // return Ok(snailquote::unescape(&string).unwrap());
    // return Ok(string.replace("\"",'\\\\\"'));
    Ok(string)
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

pub fn from_bytes(
    // resp: &Response<Body>,
    bytes: Bytes,
    content_type: &ContentType, // TODO: Convert so salvo MIME types
) -> Result<MediaContainerWrapper<MediaContainer>> {
    // const UTF8_BOM: &[u8] = &[0xef, 0xbb, 0xbf];

    // dbg!(&body);
    // let body_string = body_to_string(body).await?;
    // body.
    // let bytes = hyper::body::to_bytes(response.into_body()).await?
    // body.
    // let t: BytesMut = body.into();
    // let bytes = hyper::body::to_bytes(body).await?;
    // let string = String::from_utf8(bytes.to_vec())?;
    // dbg!(&string);
    // bytes = EventReader::new(bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes));
    // dbg!(&bytes);
    // yaserde::de::from_r
    // dbg!(&bytes);
    debug!("Response bytes: {:?}", bytes);
    let result: MediaContainerWrapper<MediaContainer> = match content_type {
        ContentType::Json => {
            // let mut c: MediaContainerWrapper<MediaContainer> =
            //     serde_json::from_str(&string).unwrap();

            let mut c: MediaContainerWrapper<MediaContainer> =
                serde_json::from_reader(&*bytes).unwrap();
            c.content_type = ContentType::Json;
            c
        }
        ContentType::Xml => MediaContainerWrapper {
            // media_container: from_xml_str(&body_string).unwrap(),
            media_container: yaserde::de::from_reader(&*bytes).unwrap(),
            content_type: ContentType::Xml,
        },
    };
    // result.content_type = content_type;
    Ok(result)
}

pub async fn from_body(
    // resp: &Response<Body>,
    body: hyper::Body,
    content_type: &ContentType,
) -> Result<MediaContainerWrapper<MediaContainer>> {
    // const UTF8_BOM: &[u8] = &[0xef, 0xbb, 0xbf];

    // dbg!(&body);
    // let body_string = body_to_string(body).await?;

    // let bytes = hyper::body::to_bytes(response.into_body()).await?
    let bytes = hyper::body::to_bytes(body).await?;
    // let string = String::from_utf8(bytes.to_vec())?;
    // dbg!(&string);
    // bytes = EventReader::new(bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes));
    // dbg!(&bytes);
    // yaserde::de::from_r
    // dbg!(&bytes);
    debug!("Response bytes: {:?}", bytes);
    let result: MediaContainerWrapper<MediaContainer> = match content_type {
        ContentType::Json => {
            // let mut c: MediaContainerWrapper<MediaContainer> =
            //     serde_json::from_str(&string).unwrap();

            let mut c: MediaContainerWrapper<MediaContainer> =
                serde_json::from_reader(&*bytes).unwrap();
            c.content_type = ContentType::Json;
            c
        }
        ContentType::Xml => MediaContainerWrapper {
            // media_container: from_xml_str(&body_string).unwrap(),
            media_container: yaserde::de::from_reader(&*bytes).unwrap(),
            content_type: ContentType::Xml,
        },
    };
    // result.content_type = content_type;
    Ok(result)
}

pub async fn from_response_hyper(resp: hyper::Response<hyper::Body>) -> Result<MediaContainerWrapper<MediaContainer>> {
    let (parts, body) = resp.into_parts();
    // let f = body.to_bytes();
    // let r = to_bytes(body).await.unwrap();

    let content_type = get_content_type_from_headers(&parts.headers);
    // let yo = body;

    let result = match from_body(body, &content_type).await {
        Ok(result) => result,
        Err(error) => {
            error!("Problem deserializing: {:?}", error);
            let container: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper::default();
            container // TOOD: Handle this higher up
        }
    };
    Ok(result)
    // let result = from_body(body, &content_type).await;
    // result
}

// Nice example of extracting response by content type: https://github.com/salvo-rs/salvo/blob/7122c3c009d7b94e7ecf155fb096f11884a8c01b/crates/core/src/test/response.rs#L47
// TODO: use body not string
pub async fn from_response(
    mut res: SalvoResponse,
) -> Result<MediaContainerWrapper<MediaContainer>> {
    // let content_type = get_content_type_from_headers(res.headers_mut());
    let content_type = res.content_type().unwrap();
    // let bytes = res.take_bytes(res.content_type().as_ref()).await.unwrap();
    let string = res.take_string().await.unwrap();
    // dbg!(&res);

    // let result = match from_bytes(bytes, &content_type) {
    let result = match from_string(string, content_type) {
        Ok(result) => result,
        Err(error) => {
            error!("Problem deserializing: {:?}", error);
            let container: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper::default();
            container // TOOD: Handle this higher up
        }
    };
    Ok(result)
}

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

// pub fn get_header_or_param(name: String, req: &Request<Body>) -> Option<String> {
//     // fn create_client_from_request(req: Request<Body>) -> Result<plex_api::HttpClient> {
//     let headers = req.headers();
//     // dbg!(req.uri().to_string());
//     // let params: HashMap<String, String> = HashMap::new();

//     let params: HashMap<String, String> = match req.uri().query() {
//         Some(v) => url::form_urlencoded::parse(v.as_bytes())
//             .into_owned()
//             .map(|v| (v.0.to_lowercase(), v.1))
//             .collect(),
//         None => HashMap::new(),
//     };
//     // let params: HashMap<String, String> =
//     //     url::form_urlencoded::parse(req.uri().query().unwrap().as_bytes())
//     //         .into_owned()
//     //         .map(|v| (v.0.to_lowercase(), v.1))
//     //         .collect();

//     // dbg!(&params);
//     let name = name.to_lowercase();
//     let val: Option<String> = match headers.get(&name) {
//         None => params.get(&name).cloned(),
//         Some(value) => Some(value.to_str().unwrap().to_string()),
//     };
//     val
// }

// pub async fn debug_resp_body(resp: Response<Body>) {
//     let (_parts, body) = resp.into_parts();
//     debug!("{:#?}", body_to_string(body).await);
// }

// pub fn clone_req(req: &Request<Body>) -> Request
