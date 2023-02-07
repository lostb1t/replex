extern crate pretty_env_logger;
use anyhow::Result;
use cached::proc_macro::cached;
use http::Method;
use http::{HeaderMap, HeaderValue};

// #[macro_use] extern crate log;
use plex_api::HttpClientBuilder;
use serde::{Deserialize, Serialize};
// use serde_json::Result;
use serde_xml_rs::from_str as from_xml_str;
use serde_xml_rs::to_string as to_xml_str;
use serde_xml_rs::from_reader as from_xml_reader;
// use quick_xml::de::from_str as from_xml_str;
// use quick_xml::se::to_string as to_xml_str;
// use serde_xml_rs::from_reader

use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::net::IpAddr;
use std::{convert::Infallible, net::SocketAddr};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct MetaData {
    rating_key: String,
    key: String,
    guid: String,
    r#type: String,
    title: String,
    thumb: String,
    art: Option<String>,
    year: Option<i32>,
    promoted: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct Hub {
    key: String,
    hub_key: Option<String>,
    title: String,
    hub_identifier: String,
    context: String,
    r#type: String,
    size: i32,
    more: bool,
    style: String,
    promoted: Option<bool>,
    // #[serde(rename = "Metadata")]
    // metadata: Option<Vec<MetaData>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct MediaContainer {
    pub size: Option<usize>,
    pub allow_sync: Option<bool>,
    pub identifier: Option<String>,
    #[serde(rename = "librarySectionID")]
    pub library_section_id: Option<u32>,
    pub library_section_title: Option<String>,
    #[serde(rename = "librarySectionUUID")]
    pub library_section_uuid: Option<String>,
    #[serde(rename = "Hub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hub: Option<Vec<Hub>>,
    #[serde(rename = "Metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Vec<MetaData>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct MediaContainerWrapper<T> {
    #[serde(rename = "MediaContainer")]
    // #[serde(rename="$value")]
    pub media_container: T,
}

// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
// #[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
// #[serde(rename_all = "camelCase")]
// pub struct MediaContainerWrapperXML<MediaContainer> {
// }

// pub struct MediaContainerWrapper<MediaContainer> {
// }

// struct PlexHttpClient {
//     pub api_url: String,
//     pub x_plex_client_identifier: String,
//     pub x_plex_token: String,
// }

// impl PlexHttpClient {
//     fn get(path: String) -> () {

//         //let json: MediaContainerWrapper<MediaContainer> = reqwest::get("http://httpbin.org/ip")?.json()?;
//     }

//     // pub fn set_api_url(self, api_url: String) -> Self
//     // {
//     //     Self {
//     //         client: self.client.and_then(move |mut client| {
//     //             client.api_url = Uri::try_from(api_url).map_err(Into::into)?;
//     //             Ok(client)
//     //         }),
//     //     }
//     // }
// }

#[cached(time = 360)]
async fn get_custom_collections(token: String) -> Vec<MetaData> {
    // TODO: Use hyper Client or hyper_reverse_proxy
    let client = HttpClientBuilder::default()
        .set_api_url("https://plex.sjoerdarendsen.dev")
        .set_x_plex_token(token)
        //.set_x_plex_client_identifier("etz23lqlxhsdinb7hv9uiu38".to_owned())
        .build()
        .expect("wut went wrong");

    let movie_collection_container: MediaContainerWrapper<MediaContainer> = client
        .get("/library/sections/1/collections")
        .json()
        .await
        .unwrap();
    let show_collection_container: MediaContainerWrapper<MediaContainer> = client
        .get("/library/sections/3/collections")
        .json()
        .await
        .unwrap();

    [
        show_collection_container.media_container.metadata.unwrap(),
        movie_collection_container.media_container.metadata.unwrap(),
    ]
    .concat()
}

// fn get_content_type_from_response(resp: &Response<Body>) -> ContentType {
fn get_content_type_from_headers(headers: &HeaderMap<HeaderValue>) -> ContentType {
    //let wut = resp.headers().get("content-type").unwrap().
    // let content_type = resp
    //     .headers()
    //     .get("content-type")
    //     .unwrap()
    //     .to_str()
    //     .unwrap();
    let content_type = headers.get("content-type").unwrap().to_str().unwrap();
    match content_type {
        "application/json" => ContentType::Json,
        "text/xml;charset=utf-8" => ContentType::Xml,
        _ => ContentType::Xml,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ContentType {
    Json,
    Xml,
}

async fn body_to_string(body: Body) -> Result<String> {
    let body_bytes = hyper::body::to_bytes(body).await?;
    let string = String::from_utf8(body_bytes.to_vec())?;
    // return Ok(snailquote::unescape(&string).unwrap());
    // return Ok(string.replace("\"",'\\\\\"'));
    Ok(string)
}

async fn from_body(
    // resp: &Response<Body>,
    body: Body,
    content_type: &ContentType,
) -> Result<MediaContainerWrapper<MediaContainer>> {
    // println!("original Response body: {:#?}", content_type);
    let body_string = body_to_string(body).await?;
    //println!("original Response body: {:#?}", body_string);
    // let body_string = std::fs::read_to_string("test/hubs2.xml").unwrap();
    // println!("original Response body: {:#?}", body_string);
    let result: MediaContainerWrapper<MediaContainer> = match content_type {
        ContentType::Json => serde_json::from_str(&body_string).unwrap(),
        ContentType::Xml => MediaContainerWrapper {
            media_container: from_xml_str(&body_string).unwrap(),
        },
    };
    Ok(result)
}

// https://stackoverflow.com/questions/73514727/return-a-hyperbody-of-serdevalue
async fn to_string(
    container: MediaContainerWrapper<MediaContainer>,
    content_type: &ContentType,
) -> Result<String> {
    match content_type {
        ContentType::Json => Ok(serde_json::to_string(&container).unwrap()),
        // ContentType::Xml => Ok("".to_owned()),
        ContentType::Xml => Ok(to_xml_str(&container.media_container).unwrap()),
    }
}

// fn create_proxied_response(mut resp: Response<Body>, body: Body) -> Response<Body> {
//     // *response.headers_mut() = remove_hop_headers(response.headers());
//     *resp.body_mut() = body;
//     resp
// }

async fn handle(client_ip: IpAddr, mut req: Request<Body>) -> Result<Response<Body>> {
    // Default is gzip. Dont want that
    req.headers_mut()
        .insert("Accept-Encoding", HeaderValue::from_static("identity"));
    println!("Proxy req: {:#?}", req);
    let uri = req.uri_mut().to_owned();
    let method = req.method_mut().to_owned();
    let token = req
        .headers()
        .get("x-plex-token")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    // let xml_string = std::fs::read_to_string("test/hubs.xml").unwrap();
    // println!("{:#?}", xml_string);
    match hyper_reverse_proxy::call(client_ip, "http://100.91.35.113:32400", req).await {
        Ok(resp) => {
            // return Ok(resp);
            if uri.path().starts_with("/hubs/sections") && method == Method::GET {
                let (mut parts, body) = resp.into_parts();
                let content_type = get_content_type_from_headers(&parts.headers);
                let mut container = from_body(body, &content_type).await?;
                println!("original Response body: {:#?}", container);
                container = patch_hubs(container, token).await;


                let body_string = to_string(container, &content_type).await?;
                let transformed_body = Body::from(body_string);
                parts.headers.remove("content-length");
                parts
                    .headers
                    .insert("x-plex-proxy", HeaderValue::from_static("true"));
                let transformed_response = Response::from_parts(parts, transformed_body);

                println!("transformed Response: {:#?}", transformed_response);
                println!("----------");
                return Ok(transformed_response);
            }
            Ok(resp)
        }
        Err(_error) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap()),
    }
}

#[tokio::main]
async fn main() {
    // TODO: support websockets
    let bind_addr = "0.0.0.0:3001";
    let addr: SocketAddr = bind_addr.parse().expect("Could not parse ip:port.");

    let make_svc = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr().ip();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(remote_addr, req))) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Running server on {:?}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn patch_hubs(
    mut container: MediaContainerWrapper<MediaContainer>,
    token: String,
) -> MediaContainerWrapper<MediaContainer> {
    if container.media_container.hub.is_none() {
        // nothing todo
        return container;
    }

    let collections = container.media_container.hub.unwrap();
    // println!("{:#?}", hub_collections.len());

    let custom_collections = get_custom_collections(token).await;

    let custom_collections_keys: Vec<String> =
        custom_collections.iter().map(|c| c.key.clone()).collect();

    let new_collections: Vec<Hub> = collections
        .into_iter()
        .filter(|c| {
            c.context != "hub.custom.collection" || custom_collections_keys.contains(&c.key)
        })
        .collect();

    // println!("{:#?}", new_collections.len());

    let size = new_collections.len();
    container.media_container.hub = Some(new_collections);
    container.media_container.size = Some(size);
    container
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(test)]
    use pretty_assertions::assert_eq;

    // #[test]
    // fn generic_test() {
    //     let json_string = std::fs::read_to_string("test/hubs.json").unwrap();
    //     let result: MediaContainerWrapper<MediaContainer> =
    //         serde_json::from_str(&json_string).unwrap();
    //     // println!("{:?}", result);
    //     println!("{:#?}", result);
    //     let entry: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper {
    //         media_container: MediaContainer {
    //             size: Some(11),
    //             identifier: Some("com.plexapp.plugins.library".to_owned()),
    //             library_section_id: Some(1),
    //             allow_sync: Some(false),
    //             library_section_title: Some("emty".to_owned()),
    //             library_section_uuid: Some("emty".to_owned()),
    //             hub: Some(vec![]),
    //             metadata: Some(vec![]),
    //         },
    //     };
    //     assert_eq!(entry, result);
    // }

    #[test]
    fn xml_test() {
        let xml_string = std::fs::read_to_string("test/hubs.xml").unwrap();
        // let result: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper {
        //     media_container: from_xml_str(&xml_string).unwrap(),
        // };
        let result: MediaContainer = from_xml_str(&xml_string).unwrap();
        // println!("{:?}", result);
        println!("{:#?}", result);
        // let entry: MediaContainer = MediaContainerWrapper {
        //     media_container: MediaContainer {
        //         size: 11,
        //         identifier: Some("com.plexapp.plugins.library".to_owned()),
        //         library_section_id: Some(1),
        //         allow_sync: false,
        //         library_section_title: Some("emty".to_owned()),
        //         library_section_uuid: Some("emty".to_owned()),
        //         hub: Some(vec![]),
        //         metadata: Some(vec![]),
        //     },
        // };
        // assert_eq!(entry, result);
    }
}
