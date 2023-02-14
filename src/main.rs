extern crate pretty_env_logger;
#[macro_use]
extern crate tracing;
use anyhow::Result;
use cached::proc_macro::cached;
use http::header::HOST;
use http::Method;
use http::{HeaderMap, HeaderValue};
use itertools::Itertools;
use std::str::FromStr;
use strum_macros::Display as EnumDisplay;
use strum_macros::EnumString;
use tracing_subscriber::FmtSubscriber;

// #[macro_use] extern crate log;
use plex_api::HttpClientBuilder;

use url::Host;
use yaserde::de::from_str as from_xml_str;
use yaserde::ser::to_string as to_xml_str;

use hyper;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper_reverse_proxy::ReverseProxy;
use hyper_trust_dns::{RustlsHttpsConnector, TrustDnsHttpConnector, TrustDnsResolver};
use std::collections::HashMap;
use std::net::IpAddr;
use std::{convert::Infallible, net::SocketAddr};

use plex_proxy::models::*;

lazy_static::lazy_static! {
    static ref PROXY_CLIENT: ReverseProxy<TrustDnsHttpConnector> = {
        ReverseProxy::new(
            hyper::Client::builder().build::<_, hyper::Body>(TrustDnsResolver::default().into_http_connector()),
        )
    };
}

async fn get_collections(server: &plex_api::Server) -> Result<Vec<MetaData>> {
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
    time = 720,
    key = "String",
    convert = r#"{ server.client().x_plex_token().to_string() }"#
)]
async fn get_cached_collections(server: &plex_api::Server) -> Vec<MetaData> {
    get_collections(&server).await.unwrap()
}

// fn get_content_type_from_response(resp: &Response<Body>) -> ContentType {
fn get_content_type_from_headers(headers: &HeaderMap<HeaderValue>) -> ContentType {
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

#[derive(Debug, Clone, PartialEq, Eq, EnumString, EnumDisplay)]
enum ContentType {
    #[strum(serialize = "application/json")]
    Json,
    #[strum(serialize = "text/xml;charset=utf-8")]
    Xml,
}

// TODO: Make this traits of the Hub struct
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

async fn from_response(
    resp: Response<Body>,
) -> Result<MediaContainerWrapper<MediaContainer>> {
    let (parts, body) = resp.into_parts();
    let content_type = get_content_type_from_headers(&parts.headers);
    from_body(body, &content_type).await
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

fn get_header_or_param(name: String, req: &Request<Body>) -> Option<String> {
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

fn create_client_from_request(req: &Request<Body>) -> Result<plex_api::HttpClient> {
    // TODO: make this a generic function ( get_value or something )
    let token: String = get_header_or_param("x-plex-token".to_string(), req).unwrap();
    let client_identifier: String =
        get_header_or_param("x-plex-client-identifier".to_string(), req).unwrap();
    // let client_identifier: String = match headers.get("x-plex-client-identifier") {
    //     None => params.get("X-Plex-Client-Identifier").unwrap().to_string(),
    //     Some(value) => value.to_str().unwrap().to_string(),
    // };

    let mut client = HttpClientBuilder::default()
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
    // TODO: Strip tokens from logging
    debug!("Proxy {:#?}", req);

    // Default is gzip. Dont want that
    req.headers_mut()
        .insert("Accept-Encoding", HeaderValue::from_static("identity"));

    // http 1 has no host so get from headers
    let host = req.headers_mut().get(HOST).unwrap().to_owned();
    let uri_parts = req.uri().clone().into_parts();
    let authority: http::uri::Authority = host.to_str().unwrap().parse().unwrap();
    *req.uri_mut() = hyper::Uri::builder()
        // .scheme(uri_parts.scheme.unwrap())
        .scheme("http") // dafuq is scheme emprty from original?
        .authority(authority)
        .path_and_query(uri_parts.path_and_query.unwrap())
        .build()
        .unwrap();

    let disable = req.headers().get("x-plex-proxy-disable").is_some();
    let uri = req.uri_mut().to_owned();
    let method = req.method_mut().to_owned();
    let mut req_copy = clone_req(&req).await;
    let mut resp_headers = HeaderMap::new();
    // let (req_parts, _) = req_copy.into_parts();

    if uri.path().starts_with("/hubs")
        && !uri.path().contains("/manage")
        && method == Method::GET
        && !disable
    {
        debug!("Mangling request");

        // let (req_parts, _) = req.into_parts();
        let content_type = get_content_type_from_headers(req_copy.headers_mut());
        let client = create_client_from_request(&req_copy).expect("Expected client");
        if !&req_copy
            .headers()
            .contains_key(http::header::ACCESS_CONTROL_ALLOW_ORIGIN)
            && req_copy.headers().contains_key(http::header::ORIGIN)
        {
            resp_headers.insert(
                http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                req_copy.headers().get(http::header::ORIGIN).unwrap().into(),
            );
        }
        // let origin_header = req_copy.headers().get("Origin");

        let mut container = MediaContainerWrapper::default();
        let mut resp: Response<Body> = Response::default();

        // TODO: Move to own function
        if uri.path().starts_with("/hubs/promoted") {
            let content_directory_id =
                get_header_or_param("contentDirectoryID".to_string(), &req_copy)
                    .expect("Expected contentDirectoryID to be set");

            container = mangle_hubs_promoted(req_copy, client_ip, content_directory_id)
                .await
                .expect("something wrong");

            let resp = Response::builder()
                .status(StatusCode::OK)
                // .header("content-rtpe", value)
                .body(Body::empty())
                .unwrap();
        } else {
            let resp = match PROXY_CLIENT
                .call(client_ip, "http://100.91.35.113:32400", req)
                .await
            {
                Ok(resp) => resp,
                Err(_error) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap(),
            };

            // let mut container = from_body(body, &content_type).await?;
            container = from_response(resp).await?;
        }

        let server = get_server(client).await?;
        container = mangle_hubs_permissions(container, &server)
            .await
            .expect("mmm");

        let body_string = to_string(container, &content_type).await?;
        let transformed_body = Body::from(body_string);
        let (mut parts, _) = resp.into_parts();
        parts.headers.remove("content-length");
        parts
            .headers
            .insert("x-plex-proxy", HeaderValue::from_static("true"));
        parts.headers.insert(
            http::header::CONTENT_TYPE,
            content_type.to_string().parse().unwrap(),
        );
        parts.headers.extend(resp_headers);

        let wrangled_dresponse = Response::from_parts(parts, transformed_body);

        // debug!(resp_headers);
        // dbg!(resp_headers);
        // debug!("Wrangled {:#?}", wrangled_dresponse);
        // println!("transformed Response: {:#?}", transformed_response);
        // println!("----------");
        return Ok(wrangled_dresponse);
    }
    Ok(PROXY_CLIENT
        .call(client_ip, "http://100.91.35.113:32400", req)
        .await
        .unwrap())
}

struct MixedPrmotedHub {}

async fn remove_param(req: Request<Body>, param: String) -> Request<Body> {
    let (mut parts, body) = req.into_parts();
    let uri: &http::Uri = &parts.uri;
    let url = url::Url::parse(&uri.to_string()).unwrap();
    let query = url.query_pairs().filter(|(name, _)| name.ne(&param));

    let mut url = url.clone();
    url.query_pairs_mut().clear().extend_pairs(query);
    let uri = hyper::Uri::from_str(&url.to_string()).unwrap();
    let mut request = Request::from_parts(parts, body);
    *request.uri_mut() = uri;
    request
}

async fn clone_req(mut req: &Request<Body>) -> Request<Body> {
    let mut request = Request::new(Body::empty());
    *request.headers_mut() = req.headers().clone();
    *request.uri_mut() = req.uri().clone();
    *request.method_mut() = req.method().clone();
    request
}

// TODO: Enable cache
// #[cached(time = 720, key = "String", convert = r#"{ req.headers().get("x-plex-token").unwrap().to_string() }"#)]
#[instrument]
async fn get_promoted_hubs(
    client_ip: IpAddr,
    mut req: Request<Body>,
) -> Result<MediaContainerWrapper<MediaContainer>> {
    // async fn get_promoted_hubs(server: &Request<Body>) {
    // let mut resp: MediaContainerWrapper<MediaContainer> = server
    // .client()
    // .get(format!("/hubs/promoted", library.id()))
    // .json()
    // .await?;
    debug!("Getting promoted hubs");
    let req = remove_param(req, "contentDirectoryID".to_owned()).await;
    // req.headers_mut().remove("contentDirectoryID");
    trace!("Proxy call {:#?}", req);
    let mut resp = PROXY_CLIENT
        .call(client_ip, "http://100.91.35.113:32400", req)
        .await
        .unwrap();
    trace!("Got {:#?}", resp);
    // client = create_client_from_request("");
    // let hubs = from_response(resp);
    from_response(resp).await
    // match hyper_reverse_proxy::call(client_ip, "http://100.91.35.113:32400", req).await {
    //     Ok(resp) => return vec![],
    //     Err(error) => error
    // }
}

// async fn mangle_hubs_promoted(
//     req: Request<Body>,
//     client_ip: IpAddr,
//     content_directory_id: String,
// ) -> Result<MediaContainerWrapper<MediaContainer>> {
//     // if config is mangle bla bla
//     // let custom_collections = get_cached_collections(server).await;
//     // if container.

//     // pinnedContentDirectoryID: 1,6,3,2
//     // let mix_collections: Vec<Hub> = collections
//     //     .into_iter()
//     //     .filter(|c| {
//     //         c.context != "hub.custom.collection" || custom_collections_keys.contains(&c.key)
//     //     })
//     //     .collect();
//     // .iter()
//     // .filter(|x| x.id == 20)
//     // .next();
//     // let mut mangled_collections = collections;
//     // mangled_collections[0].r#type = "mixed".to_string();
//     // dbg!(&content_directory_id);

//     // TODO: Dont make this hardcoded just get the first value of pinnedContentDirectoryID
//     let mut container = MediaContainerWrapper::default();
//     if content_directory_id == "1" {
//         container = get_promoted_hubs(client_ip, req).await?;
//     }

//     // lets get everything into

//     let size = container.media_container.hub.len();
//     // container.media_container.hub = mangled_collections;
//     container.media_container.size = Some(size.try_into().unwrap());
//     trace!("mangled promoted container {:#?}", container);
//     Ok(container)
// }

async fn mangle_hubs_promoted(
    req: Request<Body>,
    client_ip: IpAddr,
    content_directory_id: String,
) -> Result<MediaContainerWrapper<MediaContainer>> {
    // TODO: Dont make this hardcoded just get the first value of pinnedContentDirectoryID
    let mut container: MediaContainerWrapper<MediaContainer> =
        MediaContainerWrapper::default();
    if content_directory_id == "1" {
        container = get_promoted_hubs(client_ip, req).await?;
    }

    // for hub in &container.media_container.hub {
    //     for item in hub.metadata {
    //         dbg!(item);
    //     }
    //     // dbg!(hub);
    // }

    let collections = container.media_container.hub;
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
                new_collections[v].set_children(c
                    .into_iter()
                    .interleave(hub.get_children())
                    .collect::<Vec<MetaData>>());
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
    container.media_container.size = Some(size.try_into().unwrap());
    // trace!("mangled promoted container {:#?}", container);
    container.media_container.hub = new_collections;
    Ok(container)
}

// TODO: Should take request_containers and allowed_containers. Getting containers should be done in parent
async fn mangle_hubs_permissions(
    mut container: MediaContainerWrapper<MediaContainer>,
    server: &plex_api::Server,
) -> Result<MediaContainerWrapper<MediaContainer>> {
    // if container.media_container.hub.is_none() {
    //     // nothing todo
    //     return container;
    // }

    let collections = container.media_container.hub;
    // println!("{:#?}", hub_collections.len());

    let custom_collections = get_cached_collections(&server).await;

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
    container.media_container.hub = new_collections;
    container.media_container.size = Some(size.try_into().unwrap());
    Ok(container)
}

#[tokio::main]
async fn main() {
    // pretty_env_logger::init();
    // let subscriber = FmtSubscriber::builder()
    //     // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
    //     // will be written to stdout.
    //     .with_max_level(Level::TRACE)
    //     // completes the builder.
    //     .finish();

    // tracing::subscriber::set_global_default(subscriber)
    //     .expect("setting default subscriber failed")
    tracing_subscriber::fmt::init();
    // TODO: support websockets
    let bind_addr = "0.0.0.0:3001";
    let addr: SocketAddr = bind_addr.parse().expect("Could not parse ip:port.");

    let make_svc = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr().ip();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(remote_addr, req))) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Running server on {:?}", addr);
    debug!("started");
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
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
