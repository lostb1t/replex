#[macro_use]
extern crate tracing;
#[macro_use]
extern crate axum_core;

use axum::response::IntoResponse;
use axum::{
    extract::Path,
    extract::State,
    extract::TypedHeader,
    // http::{uri::Uri, Request, Response},
    routing::get,
    Router,
};
use axum::headers::ContentType as HContentType;
// use axum::headers::ContentType;
use cached::proc_macro::cached;
// use bytes::Bytes;
// use crate::{
//     body::{Bytes, HttpBody},
//     extract::{rejection::*, FromRequest},
//     BoxError,
// };
use http::{Request, Response};

use hyper::{client::HttpConnector, Body};

use itertools::Itertools;
use plex_proxy::models::*;
use plex_proxy::plex_client::*;
use plex_proxy::proxy::*;
use plex_proxy::utils::*;
use tower_http::cors::AllowOrigin;
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;

use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

type Client = hyper::client::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();
    // tokio::spawn(server());
    // let bla = Client::new();
    // let plex_api_client = create_client_from_request()
    let proxy = Proxy {
        host: "http://100.91.35.113:32400".to_string(),
        client: Client::new(),
    };

    let app = Router::new()
        .route("/hubs/promoted", get(get_hubs_promoted))
        .route("/hubs/sections/:id", get(get_hubs_sections))
        .route(
            "/plex_proxy/library/collections/:ids/children",
            get(get_collections_children),
        )
        .route("/*path", get(default_handler)) // catchall
        .route("/", get(default_handler))
        .with_state(proxy)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .layer(
            CorsLayer::new().allow_origin(AllowOrigin::mirror_request()), // TODO: Limit to https://app.plex.tv
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    println!("reverse proxy listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn default_handler(State(proxy): State<Proxy>, req: Request<Body>) -> Response<Body> {
    proxy.request(req).await.unwrap()
}

async fn get_hubs_sections(
    State(mut proxy): State<Proxy>,
    req: Request<Body>,
) -> MediaContainerWrapper<MediaContainer> {
    // proxy.set_plex_api_from_request(&req).await;
    // let resp = proxy.request(req).await.unwrap();
    let plex = PlexClient::from(&req);
    let resp = proxy.request(req).await.unwrap();
    // let container =
    //     MediaContainerWrapper::<MediaContainer>::from_response(proxy.request(req).await.unwrap()).unwrap();
    let mut container = from_response(resp).await.unwrap();
    container = container.fix_permissions(plex).await;
    container
}

// fn bla(req: Request<Body>) -> String {
//     "hoszaaa".to_string()
// }

// #[cached(    DOESNT WORK BECAUSE OF ARGUMENT State(mut proxy): State<Proxy>
//     time = 720,
//     key = "String",
//     convert = r#"{ bla(&req) }"#
//     // convert = r#"{ format!("{}{}", proxy.host, req.method()) }"#
// )]
async fn get_hubs_promoted(
    State(mut proxy): State<Proxy>,
    mut req: Request<Body>,
) -> MediaContainerWrapper<MediaContainer> {
    let dir_id = get_header_or_param("contentDirectoryID".to_owned(), &req).unwrap();
    let pinned_id_header =
        get_header_or_param("pinnedContentDirectoryID".to_owned(), &req).unwrap();
    let pinned_ids: Vec<&str> = pinned_id_header.split(',').collect();
    // dbg!(pinned_ids);
    //pinnedContentDirectoryID

    if dir_id != pinned_ids[0] {
        // We only fill the first one.
        //let content_type = get_content_type(req);
        return MediaContainerWrapper::default();
    }

    req = remove_param(req, "contentDirectoryID");

    let plex = PlexClient::from(&req);
    // TODO: This one can be cached globally for everybody (make sure to exclude continue watching)
    let resp = proxy.request(req).await.unwrap();
    let mut container = from_response(resp).await.unwrap();
    container.media_container.metadata = vec![];
    // dbg!(&container);
    container = container.fix_permissions(plex).await;
    container.make_mixed()
}

async fn get_collections_children(
    State(mut proxy): State<Proxy>,
    Path(ids): Path<String>,
    req: Request<Body>,
) -> MediaContainerWrapper<MediaContainer> {
    let collection_ids: Vec<u32> = ids.split(',').map(|v| v.parse().unwrap()).collect();
    let plex = PlexClient::from(&req);

    let mut children: Vec<MetaData> = vec![];
    for id in collection_ids {
        let mut c = plex.get_collection_children(id).await.unwrap();
        match children.is_empty() {
            False => {
                children = children.into_iter()
                .interleave(c.media_container.children())
                .collect::<Vec<MetaData>>();
            }
            True => children.append(&mut c.media_container.children()),
        }
        // children.append(&mut c.media_container.children())
    }
    let mut container: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper::default();
    // dbg!(req.headers().get("Accept").unwrap());
    container.content_type = get_content_type_from_headers(req.headers());
    dbg!(&container.content_type);
    let size = children.len();
    container.media_container.metadata = children;
    container.media_container.size = Some(size.try_into().unwrap());
    // container = container.make_mixed();
    container
}

// impl Clone for Request<T> {
//     fn clone(&self) -> Request<Body> {
//         let (mut parts, _) = self.into_parts();
//         Request::from_parts(parts, Body::empty())
//     }
// }
