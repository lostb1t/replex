#[macro_use]
extern crate tracing;
#[macro_use]
extern crate axum_core;
use anyhow::Result;
use axum::response::IntoResponse;
use axum::{
    body::HttpBody,
    // body::Bytes,
    extract,
    extract::State,
    // http::{uri::Uri, Request, Response},
    routing::get,
    Router,
};
// use bytes::Bytes;
// use crate::{
//     body::{Bytes, HttpBody},
//     extract::{rejection::*, FromRequest},
//     BoxError,
// };
use http::{uri::Uri, Request, Response};
use hyper::client::connect::Connect;
use hyper::{client::HttpConnector, Body};
use plex_api::HttpClientBuilder;
use plex_proxy::models::*;
use plex_proxy::proxy::*;
use plex_proxy::response::*;
use plex_proxy::utils::*;
use reqwest::RequestBuilder;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::{error::Error, net::SocketAddr};
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
        plex_api: None, // plex_api: plex_api::Server::new(
                        //     "http://100.91.35.113:32400",
                        //     HttpClientBuilder::default().build().unwrap(),
                        // )
                        // .await
                        // .unwrap(),
    };

    let app = Router::new()
        .route("/hubs/promoted", get(get_hubs_promoted))
        .route("/hubs/sections/:id", get(get_hubs_sections))
        .route("/*path", get(default_handler)) // catchall
        .route("/", get(default_handler))
        .with_state(proxy)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let addr = SocketAddr::from(([0, 0, 0, 0], 4000));
    println!("reverse proxy listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn default_handler(
    State(proxy): State<Proxy>,
    mut req: Request<Body>,
) -> Response<Body> {
    // proxy.set_request(req);
    proxy.request(req).await.unwrap()
}

async fn get_hubs_sections(
    State(mut proxy): State<Proxy>,
    req: Request<Body>,
) -> MediaContainerWrapper<MediaContainer> {
    proxy.set_plex_api_from_request(&req).await;
    let resp = proxy.request(req).await.unwrap();
    let mut container = from_response(resp).await.unwrap();
    container = container.fix_permissions(&proxy).await;
    container
}

async fn get_hubs_promoted(
    State(mut proxy): State<Proxy>,
    // axum::extract::Query(mut params): axum::extract::Query<HashMap<String, String>>,
    mut req: Request<Body>,
) -> MediaContainerWrapper<MediaContainer> {
    req = remove_param(req, "contentDirectoryID");
    proxy.set_plex_api_from_request(&req).await;

    let resp = proxy.request(req).await.unwrap();
    let mut container = from_response(resp).await.unwrap();
    container = container.fix_permissions(&proxy).await;
    container
}

// impl Clone for Request<T> {
//     fn clone(&self) -> Request<Body> {
//         let (mut parts, _) = self.into_parts();
//         Request::from_parts(parts, Body::empty())
//     }
// }
