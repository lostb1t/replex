#[macro_use]
extern crate tracing;
use anyhow::Result;
use axum::{
    body::HttpBody,
    extract::State,
    http::{uri::Uri, Request, Response},
    routing::get,
    Router,
};
use hyper::client::connect::Connect;
use hyper::{client::HttpConnector, Body};
use plex_proxy::models::*;
use plex_proxy::proxy::*;
use std::error::Error as StdError;
use std::{error::Error, net::SocketAddr};


type Client = hyper::client::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() {
    // tokio::spawn(server());
    // let bla = Client::new();
    let client = Proxy {
        host: "http://100.91.35.113:32400".to_string(),
        client: Client::new(),
    };

    let app = Router::new()
        .route("/hubs/promoted", get(handler_hubs_promoted))
        .route("/*path", get(default_handler)) // catchall
        .route("/", get(default_handler))
        .with_state(client);

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
    proxy.request(req).await.unwrap()
}

async fn handler_hubs_promoted(
    State(proxy): State<Proxy>,
    mut req: Request<Body>,
) -> Response<Body> {
    proxy.request(req).await.unwrap()
}

async fn server() {
    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("server listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
