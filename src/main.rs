extern crate pretty_env_logger;
use anyhow::Result;
use cached::proc_macro::cached;
use derive_more::{Add, Display, From, Into};
use http::Method;
use http::{HeaderMap, HeaderValue};

// #[macro_use] extern crate log;
use plex_api::HttpClientBuilder;

use serde::{Deserialize, Serialize};
// use serde_json::Result;
// use serde_xml_rs::from_reader as from_xml_reader;
// use serde_xml_rs::from_str as from_xml_str;
// use serde_xml_rs::to_string as to_xml_str;
// use quick_xml::de::from_str as from_xml_str;
// use quick_xml::se::to_string as to_xml_str;

use yaserde::de::from_str as from_xml_str;
use yaserde::ser::to_string as to_xml_str;
// use yaserde::de::from_str;

use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::future::Future;
use std::net::IpAddr;
use std::{convert::Infallible, net::SocketAddr};

// use plex_proxy::xml::*;
// use plex_proxy::models::*;
use plex_proxy::models::*;

// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
// #[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
// #[serde(rename_all = "camelCase")]
// pub struct MetaData {
//     rating_key: String,
//     key: String,
//     guid: String,
//     r#type: String,
//     title: String,
//     thumb: String,
//     art: Option<String>,
//     year: Option<i32>,
//     promoted: Option<bool>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
// #[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
// #[serde(rename_all = "camelCase")]
// // #[serde(flatten)]
// pub struct Hub {
//     #[serde(rename = "@key")]
//     key: String,
//     #[serde(rename = "@hubKey")]
//     hub_key: Option<String>,
//     #[serde(rename = "@title")]
//     title: String,
//     #[serde(rename = "@hubIdentifier")]
//     hub_identifier: String,
//     #[serde(rename = "@context")]
//     context: String,
//     #[serde(rename = "@type")]
//     r#type: String,
//     #[serde(rename = "@size")]
//     size: i32,
//     #[serde(rename = "@more")]
//     more: bool,
//     #[serde(rename = "@style")]
//     style: String,
//     #[serde(rename = "@promoted")]
//     promoted: Option<bool>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
// #[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
// #[serde(rename_all = "camelCase")]
// pub struct MediaContainer {
//     pub size: Option<usize>,
//     pub allow_sync: Option<bool>,
//     pub identifier: Option<String>,
//     #[serde(rename = "librarySectionID")]
//     pub library_section_id: Option<u32>,
//     pub library_section_title: Option<String>,
//     #[serde(rename = "librarySectionUUID")]
//     pub library_section_uuid: Option<String>,
//     // #[serde(skip_serializing_if = "Option::is_none")]
//     #[serde(rename = "Hub", default)]
//     pub hubs: Vec<Hub>,
//     // #[serde(rename = "Metadata")]
//     // #[serde(skip_serializing_if = "Option::is_none")]
//     // pub metadata: Option<Vec<MetaData>>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
// #[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
// #[serde(rename_all = "camelCase")]
// pub struct MediaContainerWrapper<T> {
//     #[serde(rename = "MediaContainer")]
//     // #[serde(rename="$value")]
//     pub media_container: T,
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

// #[cached(time = 360)]
// async fn get_custom_collections(token: String) -> Vec<MetaData> {
//     let client = HttpClientBuilder::default()
//         .set_api_url("https://plex.sjoerdarendsen.dev")
//         .set_x_plex_token(token)
//         //.set_x_plex_client_identifier("etz23lqlxhsdinb7hv9uiu38".to_owned())
//         .build()
//         .expect("wut went wrong");

//     client.

//     let movie_collection_container: MediaContainerWrapper<MediaContainer> = client
//         .get("/library/sections/1/collections")
//         .json()
//         .await
//         .unwrap();
//     let show_collection_container: MediaContainerWrapper<MediaContainer> = client
//         .get("/library/sections/3/collections")
//         .json()
//         .await
//         .unwrap();

//     [
//         show_collection_container.media_container.metadata.unwrap(),
//         movie_collection_container.media_container.metadata.unwrap(),
//     ]
//     .concat()
// }

// struct GenericCollection<T> {
//     plex_api::Collection<T>: plex_api::Collection<T>
//     // collection: plex_api::Collection<T>,
//     // r#type: plex_api::Item
// }

// #[derive(PartialEq, From)]
// struct GenericCollection(plex_api::Item);
// // #[cached(time = 360)]
// // async fn get_library(server: plex_api::Server) -> Result<plex_api::Library> {

// // }

// async fn get_library_collections(library: plex_api::Library) {}

// #[cached(time = 360, convert = r#"{ make_server_key(server) }"#)]

async fn get_collections(server: plex_api::Server) -> Result<Vec<MetaData>> {
    // let mut collections: Vec<GenericCollection<plex_api::Item>> = vec![];
    let mut collections = vec![];
    for library in server.libraries() {
        // library.media

        let mut resp: MediaContainerWrapper<MediaContainer> = server
            .client()
            .get(format!("/library/sections/{}/collections", library.id()))
            .json()
            .await?;
        collections.append(&mut resp.media_container.metadata);

        // Library::new
        // match library {
        //     plex_api::Library::Movie(library) => {
        //         // let c = library.collections().await.unwrap();
        //         let b = library.collections().await.unwrap().iter().map(|x| {
        //             collections.push(GenericCollection {
        //                 collection: x.clone(),
        //             })
        //         });
        //     }
        //     //     collections.append(GenericCollection {
        //     //     collection: library.collections().await.unwrap(),
        //     // })},
        //     _ => {}
        // }
        // collections = lib.collections().await.unwrap();
    }
    // println!("no cache");
    Ok(collections)
    //  let libraries = server.libraries();
    // let mut future_collections: Vec<Future>;
    // Todo: Run async
    // match library {
    // let mut collections: Vec<plex_api::Collection<plex_api::Item>> = vec![];
    //let mut collections: Vec<plex_api::Collection<T: plex_api::Item>> = vec![];
    // let mut collections: Vec<GenericCollection> = vec![];
    // for library in server.libraries() {
    //     // Library::new
    //     match library {
    //         plex_api::Library::Movie(library) => {
    //             // let c = library.collections().await.unwrap();
    //             let b = library.collections().await.unwrap().iter().map(|x| {
    //                 collections.push(GenericCollection {
    //                     collection: x
    //                 })
    //             });
    //         }
    //         //     collections.append(GenericCollection {
    //         //     collection: library.collections().await.unwrap(),
    //         // })},
    //         _ => {}
    //     }
    //     // collections = lib.collections().await.unwrap();
    // }
    //  let library = if let Library::Movie(lib) = libraries.get(0).unwrap() {
    //      lib
    //  } else {
    //      panic!("Unexpected library type");
    //  };
    //  let collections = library.collections().await.unwrap();
}

#[cached(
    time = 360,
    key = "String",
    convert = r#"{ server.client().x_plex_token().to_string() }"#
)]
async fn get_cached_collections(server: plex_api::Server) -> Vec<MetaData> {
    get_collections(server).await.unwrap()
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

fn create_client_from_request(req: &Request<Body>) -> Result<plex_api::HttpClient> {
    let headers = req.headers();
    let token = headers
        .get("x-plex-token")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let client_identifier = headers
        .get("x-plex-client-identifier")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let client = HttpClientBuilder::default()
        .set_api_url("https://plex.sjoerdarendsen.dev")
        .set_x_plex_token(token)
        .set_x_plex_client_identifier(client_identifier)
        .build()?;
    Ok(client)
}

// fn tes_client_from_request(req: &Request<Body>) {
//     println!("{:#?}", req);
// }

// TODO: should not reinitaed server everytime. Maybe cache them for each user/token.
async fn get_server(client: plex_api::HttpClient) -> Result<plex_api::Server> {
    let server: plex_api::Server =
        plex_api::Server::new("https://plex.sjoerdarendsen.dev", client).await?;
    Ok(server)
}

// TODO: Handle erors. Dont propagate here
async fn handle(client_ip: IpAddr, mut req: Request<Body>) -> Result<Response<Body>> {
    // Default is gzip. Dont want that
    req.headers_mut()
        .insert("Accept-Encoding", HeaderValue::from_static("identity"));
    // println!("Proxy req: {:#?}", req);
    let uri = req.uri_mut().to_owned();
    let method = req.method_mut().to_owned();
    // let token = req
    //     .headers()
    //     .get("x-plex-token")
    //     .unwrap()
    //     .to_str()
    //     .unwrap()
    //     .to_string();
    // let xml_string = std::fs::read_to_string("test/hubs.xml").unwrap();
    // println!("{:#?}", xml_string);
    // tes_client_from_request(&req);
    let client = create_client_from_request(&req).expect("huha");
    // let movie_collection_container: MediaContainerWrapper<MediaContainer> = client
    //     .get("/hubs/sections/1")
    //     .json()
    //     .await
    //     .unwrap();
    // println!("{:#?}", movie_collection_container);
    match hyper_reverse_proxy::call(client_ip, "http://100.91.35.113:32400", req).await {
        Ok(resp) => {
            // return Ok(resp);
            if uri.path().starts_with("/hubs") && method == Method::GET {
                let (mut parts, body) = resp.into_parts();
                let content_type = get_content_type_from_headers(&parts.headers);
                let mut container = from_body(body, &content_type).await?;
                // println!("original Response body: {:#?}", container);

                let server = get_server(client).await?;

                container = patch_hubs(container, server).await.expect("mmm");
                // println!("sup");
                let body_string = to_string(container, &content_type).await?;
                let transformed_body = Body::from(body_string);
                parts.headers.remove("content-length");
                parts
                    .headers
                    .insert("x-plex-proxy", HeaderValue::from_static("true"));
                let transformed_response = Response::from_parts(parts, transformed_body);

                // println!("transformed Response: {:#?}", transformed_response);
                // println!("----------");
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
        async move { Ok::<_, Infallible>(service_fn(move |mut req| handle(remote_addr, req))) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Running server on {:?}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn patch_hubs(
    mut container: MediaContainerWrapper<MediaContainer>,
    server: plex_api::Server,
) -> Result<MediaContainerWrapper<MediaContainer>> {
    // if container.media_container.hub.is_none() {
    //     // nothing todo
    //     return container;
    // }

    let collections = container.media_container.hub;
    // println!("{:#?}", hub_collections.len());

    let custom_collections = get_cached_collections(server).await;

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
    container.media_container.hub = new_collections;
    container.media_container.size = Some(size.try_into().unwrap());
    Ok(container)
}

#[cfg(test)]
mod tests {
    use std::result;

    use super::*;
    #[cfg(test)]
    use pretty_assertions::assert_eq;

    // #[test]
    // fn custom_collections_test() {
    //     let server = get_server();
    //     collections = get_collections();
    // }

    #[test]
    fn json_test() {
        let json_string = std::fs::read_to_string("test/hubs.json").unwrap();
        let container: MediaContainerWrapper<MediaContainer> =
            serde_json::from_str(&json_string).unwrap();
        // println!("{:?}", result);
        println!("{:#?}", container);
        let result = serde_json::to_string(&container).unwrap();
        // println!("{:?}", result);
        println!("{:#?}", result);
        // let entry: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper {
        //     media_container: MediaContainer {
        //         size: Some(11),
        //         identifier: Some("com.plexapp.plugins.library".to_owned()),
        //         library_section_id: Some(1),
        //         allow_sync: Some(false),
        //         library_section_title: Some("emty".to_owned()),
        //         library_section_uuid: Some("emty".to_owned()),
        //         hub: Some(vec![]),
        //         metadata: Some(vec![]),
        //     },
        // };
        // assert_eq!(entry, result);
    }

    #[test]
    fn xml_test() {
        let xml_string = std::fs::read_to_string("test/hubs.xml").unwrap();
        // let result: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper {
        //     media_container: from_xml_str(&xml_string).unwrap(),
        // };
        let container: MediaContainer = from_xml_str(&xml_string).unwrap();
        //let container: HubsContainer = HubsContainer{ media_container: from_xml_str(&xml_string).unwrap()};

        println!("{:#?}", container);
        let result = to_xml_str(&container).unwrap();

        println!("{:?}", result);
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
