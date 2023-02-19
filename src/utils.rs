extern crate tracing;
use anyhow::Result;
use axum::{
    http::{uri::Uri, HeaderMap, HeaderValue, Request, Response},
};

// use http::{HeaderMap, HeaderValue};
use hyper::{Body};
use strum_macros::Display as EnumDisplay;
use strum_macros::EnumString;

use yaserde::ser::to_string as to_xml_str;




use plex_api::HttpClientBuilder;
use std::collections::HashMap;

use crate::models::*;

pub fn remove_param(mut req: Request<Body>, param: &str) -> Request<Body> {
    let mut uri = pathetic::Uri::default()
        .with_path(req.uri_mut().path())
        .with_query(req.uri_mut().query());
    let query: Vec<(String, String)> = uri
        .query_pairs()
        .filter(|(name, _)| name != param)
        .map(|(name, value)| (name.into_owned(), value.into_owned()))
        .collect();
    uri.query_pairs_mut().clear().extend_pairs(query);
    *req.uri_mut() = Uri::try_from(uri.as_str()).unwrap();
    req
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString, EnumDisplay)]
pub enum ContentType {
    #[strum(serialize = "application/json")]
    Json,
    #[strum(serialize = "text/xml;charset=utf-8")]
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

pub fn get_content_type(req: Request<Body>) -> ContentType {
    let (parts, _body) = req.into_parts();
    get_content_type_from_headers(&parts.headers)
}

// TODO: Make this traits of the Hub struct
pub async fn body_to_string(body: Body) -> Result<String> {
    // dbg!(&body.boxed());
    // dbg!(&body);
    let body_bytes = hyper::body::to_bytes(body).await?;
    // let body_bytes = to_bytes(body).await.unwrap();
    
    let string = String::from_utf8(body_bytes.to_vec())?;
    dbg!(&string);
    // return Ok(snailquote::unescape(&string).unwrap());
    // return Ok(string.replace("\"",'\\\\\"'));
    Ok(string)
}

pub async fn from_body(
    // resp: &Response<Body>,
    body: Body,
    content_type: &ContentType,
) -> Result<MediaContainerWrapper<MediaContainer>> {
    // const UTF8_BOM: &[u8] = &[0xef, 0xbb, 0xbf];

    // dbg!(&body);
    // let body_string = body_to_string(body).await?;

    // let bytes = hyper::body::to_bytes(response.into_body()).await?
    let bytes = hyper::body::to_bytes(body).await?;
    // bytes = EventReader::new(bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes));
    // dbg!(&bytes);
    // yaserde::de::from_r
    let result: MediaContainerWrapper<MediaContainer> = match content_type {
        ContentType::Json => {
            // let mut c: MediaContainerWrapper<MediaContainer> =
            //     serde_json::from_str(&body_string).unwrap();
            // let mut c: MediaContainerWrapper<MediaContainer> =
                // serde_json::from_slice(&bytes).unwrap();
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

pub async fn from_response(
    resp: Response<Body>,
) -> Result<MediaContainerWrapper<MediaContainer>> {
    let (parts, body) = resp.into_parts();
    // let f = body.to_bytes();
    // let r = to_bytes(body).await.unwrap();
    // dbg!(r);
    let content_type = get_content_type_from_headers(&parts.headers);
    // let yo = body;
    from_body(body, &content_type).await
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

pub fn get_header_or_param(name: String, req: &Request<Body>) -> Option<String> {
    // fn create_client_from_request(req: Request<Body>) -> Result<plex_api::HttpClient> {
    let headers = req.headers();
    // dbg!(req.uri().to_string());
    // let params: HashMap<String, String> = HashMap::new();

    let params: HashMap<String, String> = match req.uri().query() {
        Some(v) => url::form_urlencoded::parse(v.as_bytes())
            .into_owned()
            .map(|v| (v.0.to_lowercase(), v.1))
            .collect(),
        None => HashMap::new(),
    };
    // let params: HashMap<String, String> =
    //     url::form_urlencoded::parse(req.uri().query().unwrap().as_bytes())
    //         .into_owned()
    //         .map(|v| (v.0.to_lowercase(), v.1))
    //         .collect();

    // dbg!(&params);
    let name = name.to_lowercase();
    let val: Option<String> = match headers.get(&name) {
        None => params.get(&name).cloned(),
        Some(value) => Some(value.to_str().unwrap().to_string()),
    };
    val
}

// pub fn create_client_from_request(req: &Request<Body>) -> Result<plex_api::HttpClient> {
//     // TODO: make this a generic function ( get_value or something )
//     let token: String = get_header_or_param("x-plex-token".to_string(), req).unwrap();
//     let client_identifier: String =
//         get_header_or_param("x-plex-client-identifier".to_string(), req).unwrap();
//     // let client_identifier: String = match headers.get("x-plex-client-identifier") {
//     //     None => params.get("X-Plex-Client-Identifier").unwrap().to_string(),
//     //     Some(value) => value.to_str().unwrap().to_string(),
//     // };

//     let client = HttpClientBuilder::default()
//         .set_api_url("https://plex.sjoerdarendsen.dev")
//         .set_x_plex_token(token)
//         .set_x_plex_client_identifier(client_identifier)
//         .build()?;
//     Ok(client)
// }
// async fn to_bytes<T>(body: T) -> Result<Bytes, T::Error>
// where
//     T: httpBody,
// {
//     futures_util::pin_mut!(body);

//     // If there's only 1 chunk, we can just return Buf::to_bytes()
//     let mut first = if let Some(buf) = body.data().await {
//         buf?
//     } else {
//         return Ok(Bytes::new());
//     };

//     let second = if let Some(buf) = body.data().await {
//         buf?
//     } else {
//         return Ok(first.copy_to_bytes(first.remaining()));
//     };

//     // With more than 1 buf, we gotta flatten into a Vec first.
//     let cap = first.remaining() + second.remaining() + body.size_hint().lower() as usize;
//     let mut vec = Vec::with_capacity(cap);
//     vec.put(first);
//     vec.put(second);

//     while let Some(buf) = body.data().await {
//         vec.put(buf?);
//     }

//     Ok(vec.into())
// }