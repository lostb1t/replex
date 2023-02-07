extern crate pretty_env_logger;
use anyhow::Result;
use http::{HeaderMap, HeaderValue};
// #[macro_use] extern crate log;
use http::{uri::PathAndQuery, Uri};
use plex_api::{HttpClient, HttpClientBuilder};
use serde::{Deserialize, Serialize};
// use serde_json::Result;
use serde_xml_rs::from_str as from_xml_str;
use serde_xml_rs::to_string as to_xml_str;
// use serde_xml_rs::from_reader

use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::net::IpAddr;
use std::{convert::Infallible, net::SocketAddr};
// use simple_proxy::middlewares::Logger;
// use simple_proxy::{Environment, SimpleProxy};

// // use crate::middleware::Cors;
// use plex_proxy::middleware::Cors;
// use plex_proxy::proxy::proxy;

// type Metadata struct {
// 	RatingKey             string       `json:"ratingKey"`
// 	Key                   string       `json:"key"`
// 	GUID                  string       `json:"guid"`
// 	AltGUIDs              []AltGUID    `json:"Guid,omitempty"`
// 	Studio                string       `json:"studio"`
// 	Type                  string       `json:"type"`
// 	Title                 string       `json:"title"`
// 	LibrarySectionTitle   string       `json:"librarySectionTitle"`
// 	LibrarySectionID      int          `json:"librarySectionID"`
// 	LibrarySectionKey     string       `json:"librarySectionKey"`
// 	OriginalTitle         string       `json:"originalTitle,omitempty"`
// 	ContentRating         string       `json:"contentRating"`
// 	Rating                float64      `json:"rating"`
// 	Ratings               []Rating     `json:"Rating,omitempty"`
// 	AudienceRating        float64      `json:"audienceRating"`
// 	Year                  int          `json:"year"`
// 	Tagline               string       `json:"tagline"`
// 	Thumb                 string       `json:"thumb"`
// 	Art                   string       `json:"art"`
// 	Duration              int          `json:"duration"`
// 	OriginallyAvailableAt string       `json:"originallyAvailableAt"`
// 	AddedAt               int          `json:"addedAt"`
// 	UpdatedAt             int          `json:"updatedAt"`
// 	AudienceRatingImage   string       `json:"audienceRatingImage"`
// 	ChapterSource         string       `json:"chapterSource,omitempty"`
// 	Media                 []Media      `json:"Media"`
// 	Genre                 []Genre      `json:"Genre"`
// 	Director              []Director   `json:"Director"`
// 	Writer                []Writer     `json:"Writer"`
// 	Country               []Country    `json:"Country"`
// 	Collection            []Collection `json:"Collection"`
// 	Role                  []Role       `json:"Role"`
// 	PrimaryExtraKey       string       `json:"primaryExtraKey,omitempty"`
// 	TitleSort             string       `json:"titleSort,omitempty"`
// }

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
    #[serde(rename = "Metadata")]
    metadata: Option<Vec<MetaData>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct MediaContainer {
    pub size: usize,
    pub allow_sync: bool,
    pub identifier: Option<String>,
    #[serde(rename = "librarySectionID")]
    pub library_section_id: Option<u32>,
    pub library_section_title: Option<String>,
    #[serde(rename = "librarySectionUUID")]
    pub library_section_uuid: Option<String>,
    #[serde(rename = "Hub")]
    pub hub: Option<Vec<Hub>>,
    #[serde(rename = "Metadata")]
    metadata: Option<Vec<MetaData>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct MediaContainerWrapper<T> {
    #[serde(rename = "MediaContainer")]
    pub media_container: T,
}

// impl MediaContainerWrapper<MediaContainer> {

// }

// impl Json for MediaContainerWrapper<MediaContainer> {
// }

// impl Point {
//     fn as_json(&self)-> String {
//         return serde_json::to_string(&self).unwrap()
//     }

//     fn from_json(s: &str)-> Self {
//         return serde_json::from_str(s).unwrap()
//     }
// }

// #[derive(Debug, Deserialize, Clone)]
// #[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
// #[serde(rename_all = "camelCase")]
// pub struct MediaContainerWrapper<T> {
//     #[serde(rename = "MediaContainer")]
//     pub media_container: T,
// }

struct PlexHttpClient {
    pub api_url: String,
    pub x_plex_client_identifier: String,
    pub x_plex_token: String,
}

impl PlexHttpClient {
    fn get(path: String) -> () {

        //let json: MediaContainerWrapper<MediaContainer> = reqwest::get("http://httpbin.org/ip")?.json()?;
    }

    // pub fn set_api_url(self, api_url: String) -> Self
    // {
    //     Self {
    //         client: self.client.and_then(move |mut client| {
    //             client.api_url = Uri::try_from(api_url).map_err(Into::into)?;
    //             Ok(client)
    //         }),
    //     }
    // }
}

async fn get_custom_collections() -> Vec<MetaData> {
    let client = HttpClientBuilder::default()
        .set_api_url("https://plex.sjoerdarendsen.dev")
        .set_x_plex_token("RrZN1WRwYYfao2cuiHs5".to_owned())
        .set_x_plex_client_identifier("etz23lqlxhsdinb7hv9uiu38".to_owned())
        .build()
        .expect("wut went wrong");

    // let server = Server::new("https://plex.sjoerdarendsen.dev", client)
    //     .await
    //     .unwrap();
    // let libraries = server.libraries();
    // let library = if let Library::Movie(lib) = libraries.get(0).unwrap() {
    //     lib
    // } else {
    //     panic!("Unexpected library type");
    // };
    // let collections = library.collections().await.unwrap();

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

    let collections = [
        show_collection_container.media_container.metadata.unwrap(),
        movie_collection_container.media_container.metadata.unwrap(),
    ]
    .concat();
    // println!("{:#?}", collections);

    collections
}

// #[derive(Debug, Clone)]
// pub struct Config();
// #[tokio::main]
// async fn main() {
//     // let args = Cli::from_args();

//     let mut proxy = SimpleProxy::new(3005, Environment::Development);
//     let logger = Logger::new();
//     let cors = Cors::new();
//     // let router = Router::new(&Config());

//     // Order matters
//     // proxy.add_middleware(Box::new(router));
//     proxy.add_middleware(Box::new(logger));
//     proxy.add_middleware(Box::new(cors));

//     // Start proxy
//     let _ = proxy.run().await;
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = SocketAddr::from(([100, 91, 35, 113], 32400));

//     let listener = TcpListener::bind(addr).await?;
//     println!("Listening on http://{}", addr);

//     loop {
//         let (stream, _) = listener.accept().await?;

//         tokio::task::spawn(async move {
//             if let Err(err) = http1::Builder::new()
//                 .preserve_header_case(true)
//                 .title_case_headers(true)
//                 .serve_connection(stream, service_fn(proxy))
//                 .with_upgrades()
//                 .await
//             {
//                 println!("Failed to serve connection: {:?}", err);
//             }
//         });
//     }
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     pretty_env_logger::init();

//     let in_addr: SocketAddr = ([127, 0, 0, 1], 3001).into();
//     let out_addr: SocketAddr = ([100, 91, 35, 113], 32400).into();

//     let out_addr_clone = out_addr.clone();

//     let listener = TcpListener::bind(in_addr).await?;

//     println!("Listening on http://{}", in_addr);
//     println!("Proxying on http://{}", out_addr);

//     loop {
//         let (stream, _) = listener.accept().await?;

//         let service = service_fn(move |mut req| {
//             let uri_string = format!(
//                 "http://{}{}",
//                 out_addr_clone,
//                 req.uri()
//                     .path_and_query()
//                     .map(|x| x.as_str())
//                     .unwrap_or("/")
//             );
//             // println!("{:#?}", req.uri().path());
//             let path = req.uri().path();
//             let uri = uri_string.parse().unwrap();

//             let mut is_hubs = false;
//             if req.uri().path().starts_with("/hubs") {
//                 is_hubs = true;
//             }

//             *req.uri_mut() = uri;

//             let host = req.uri().host().expect("uri has no host");
//             let port = req.uri().port_u16().unwrap_or(80);
//             let addr = format!("{}:{}", host, port);
//             // let addr = format!("plex.sjoerdarendsen.dev:443");

//             async move {
//                 let client_stream = TcpStream::connect(addr).await.unwrap();

//                 let (mut sender, conn) =
//                     hyper::client::conn::http1::handshake(client_stream).await?;
//                 tokio::task::spawn(async move {
//                     if let Err(err) = conn.await {
//                         println!("Connection failed: {:?}", err);
//                     }
//                 });

//                 let response = sender.send_request(req).await;

//                 if is_hubs {
//                     // if let Some(content_type) = response?.headers().get("content-type") {
//                     //     println!("{:#?}", content_type);

//                     // }

//                 }

//                 response
//             }
//         });

//         tokio::task::spawn(async move {
//             if let Err(err) = http1::Builder::new()
//                 .serve_connection(stream, service)
//                 .await
//             {
//                 println!("Failed to servce connection: {:?}", err);
//             }
//         });
//     }
// }

// fn debug_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
//     let body_str = format!("{:?}", req);
//     Ok(Response::new(Body::from(body_str)))
// }

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
        "application/json" => ContentType::JSON,
        "text/xml;charset=utf-8" => ContentType::XML,
        _ => ContentType::XML,
    }
}

enum ContentType {
    JSON,
    XML,
}

async fn body_to_string(body: Body) -> Result<String> {
    let body_bytes = hyper::body::to_bytes(body).await?;
    let string = String::from_utf8(body_bytes.to_vec())?;
    //let string = String::from_utf8(body_bytes.to_vec()).unwrap();
    Ok(string)
}

async fn from_body(
    // resp: &Response<Body>,
    body: Body,
    content_type: &ContentType,
) -> Result<MediaContainerWrapper<MediaContainer>> {
    //let content_type = get_content_type_from_response(&resp);
    // println!("{:#?}", resp);
    // Get the response body bytes.
    // let body_bytes = hyper::body::to_bytes(resp).await?;
    // //let body_bytes = hyper::body::to_bytes(resp.body_mut()).await.unwrap();
    // // println!("{:#?}", body_bytes);
    // // Convert the body bytes to utf-8
    // let body = String::from_utf8(body_bytes.to_vec()).unwrap();

    //let body = String::from_utf8(body_bytes.into_iter().collect()).unwrap();
    let body_string = body_to_string(body).await?;

    let result: MediaContainerWrapper<MediaContainer> = match content_type {
        ContentType::JSON => serde_json::from_str(&body_string).unwrap(),
        ContentType::XML => from_xml_str(&body_string).unwrap(),
    };
    Ok(result)
}

// https://stackoverflow.com/questions/73514727/return-a-hyperbody-of-serdevalue
async fn to_string(
    container: MediaContainerWrapper<MediaContainer>,
    content_type: &ContentType,
) -> Result<String> {
    match content_type {
        ContentType::JSON => Ok(serde_json::to_string(&container).unwrap()),
        ContentType::XML => Ok(to_xml_str(&container).unwrap()),
    }
}

// fn copy_headers(headers: &HeaderMap<HeaderValue>) -> HeaderMap<HeaderValue> {
//     let mut result = HeaderMap::new();
//     for (k, v) in headers.iter() {
//         if !is_hop_header(k.as_str()) {
//             result.insert(k.clone(), v.clone());
//         }
//     }
//     result
// }

fn create_proxied_response(mut resp: Response<Body>, body: Body) -> Response<Body> {
    // *response.headers_mut() = remove_hop_headers(response.headers());
    *resp.body_mut() = body;
    resp
}

async fn handle(client_ip: IpAddr, mut req: Request<Body>) -> Result<Response<Body>> {
    // Default is gzip. Dont want that
    req.headers_mut()
        .insert("Accept-Encoding", HeaderValue::from_static("identity"));
    let uri = req.uri_mut().to_owned();

    match hyper_reverse_proxy::call(client_ip, "http://100.91.35.113:32400", req).await {
        Ok(resp) => {
            if uri.path().starts_with("/hubs") {
                let (mut parts, body) = resp.into_parts();
                let content_type = get_content_type_from_headers(&parts.headers);
                let mut container = from_body(body, &content_type).await?;
                container = patch_hubs(container).await;
                let body_string = to_string(container, &content_type).await?;
                let transformed_body = Body::from(body_string);
                parts.headers.remove("content-length");
                let transformed_response = Response::from_parts(parts, transformed_body);
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
    //let out_addr: SocketAddr = ([100, 91, 35, 113], 32400).into();
    let bind_addr = "127.0.0.1:3001";
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
) -> MediaContainerWrapper<MediaContainer> {
    // println!("Hello, world!");

    // let result = match application_type.as_str() {
    //     // Match a single value
    //     "json" => serde_json::from_str(&s),
    //     // Match several values
    //     "xml" => serde_json::from_str(&s),
    //     _ => Ok(Err("err")),
    // };

    // let mut result: MediaContainerWrapper<MediaContainer> = serde_json::from_str(&s).unwrap();
    if container.media_container.hub.is_none() {
        // nothing todo
        return container;
    }

    let hub_collections = container.media_container.hub.unwrap();
    // return hub_collections
    println!("{:#?}", hub_collections.len());

    let custom_collections = get_custom_collections().await;

    let custom_collections_keys: Vec<String> =
        custom_collections.iter().map(|c| c.key.clone()).collect();

    let new_collections: Vec<Hub> = hub_collections
        .into_iter()
        .filter(|c| {
            c.context != "hub.custom.collection" || custom_collections_keys.contains(&c.key)
        })
        .collect();
    
    println!("{:#?}", new_collections.len());
    // let allowed_collection_keys: Vec<_> = allowed_collections
    //     .iter()
    //     .map(|c| String::from(c.key.clone()))
    //     .collect();

    // let new_collections: Vec<MetaData> = hub_collections
    //     .into_iter()
    //     .filter(|c| allowed_collection_keys.contains(&c.key))
    //     .collect();

    // println!("{:#?}", new_collections.len());
    let size = new_collections.len();
    container.media_container.hub = Some(new_collections);
    container.media_container.size = size;
    // println!("{:#?}", collection_keys);
    //serde_json::from_str(&json_string).unwrap();
    // let remotes = response.json::<serde_json::Value>().await?;
    container
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(test)]
    use pretty_assertions::assert_eq;

    #[test]
    fn generic_test() {
        let json_string = std::fs::read_to_string("test/hubs.json").unwrap();
        let result: MediaContainerWrapper<MediaContainer> =
            serde_json::from_str(&json_string).unwrap();
        // println!("{:?}", result);
        println!("{:#?}", result);
        let entry: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper {
            media_container: MediaContainer {
                size: 11,
                identifier: Some("com.plexapp.plugins.library".to_owned()),
                library_section_id: 1,
                hub: Some(vec![]),
                metadata: Some(vec![]),
            },
        };
        assert_eq!(entry, result);
    }

    #[test]
    fn xml_test() {
        let xml_string = std::fs::read_to_string("test/hubs.xml").unwrap();
        let result: MediaContainerWrapper<MediaContainer> = from_xml_str(&xml_string).unwrap();
        // println!("{:?}", result);
        println!("{:#?}", result);
        let entry: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper {
            media_container: MediaContainer {
                size: 11,
                identifier: Some("com.plexapp.plugins.library".to_owned()),
                library_section_id: 1,
                hub: Some(vec![]),
                metadata: Some(vec![]),
            },
        };
        assert_eq!(entry, result);
    }
}
