#[macro_use]
extern crate tracing;
use anyhow::Result;
use axum::{
    extract::State,
    http::{uri::Uri, Request, Response},
    routing::get,
    Router, body::HttpBody,
};
use hyper::{client::HttpConnector, Body};
use hyper::client::connect::Connect;
use std::{net::SocketAddr, error::Error};
use plex_proxy::models::*;
use std::error::Error as StdError;

// trait ProxyRequest: hyper::client::Client {
trait ProxyRequest {
    // fn proxy_req(&self) -> hyper::client::ResponseFuture;
    fn proxy_req(&self) -> String;

}

impl<C, B> ProxyRequest for hyper::client::Client<C, B> {
    fn proxy_req(&self) -> String {
        dbg!("yup").to_string()
    }
}
// impl Summary for NewsArticle {
// impl<C, B> hyper::client::Client<C, B>
// where
//     C: Connect + Clone + Send + Sync + 'static,
//     B: HttpBody + Send + 'static,
//     B::Data: Send,
//     B::Error: Into<Box<dyn StdError + Send + Sync>>,
// {
//     fn summarize(&self) -> String {
//         format!("{}, by {} ({})", self.headline, self.author, self.location)
//     }
// }

type Client = hyper::client::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() {
    // tokio::spawn(server());

    let client = Client::new();

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
    State(client): State<Client>,
    mut req: Request<Body>,
) -> Response<Body> {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let uri = format!("http://100.91.35.113:32400{}", path_query);
    dbg!(&uri);
    *req.uri_mut() = Uri::try_from(uri).unwrap();

    client.request(req).await.unwrap()
}

async fn handler_hubs_promoted(
    State(client): State<Client>,
    mut req: Request<Body>,
) -> Response<Body> {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);
    dbg!(client.proxy_req());
    let uri = format!("http://100.91.35.113:32400{}", path_query);
    *req.uri_mut() = Uri::try_from(uri).unwrap();

    client.request(req).await.unwrap()
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
