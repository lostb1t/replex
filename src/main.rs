#[macro_use]
extern crate tracing;
use anyhow::Result;
use axum::{
    extract::State,
    http::{uri::Uri, Request, Response},
    routing::get,
    Router,
};
use hyper::{client::HttpConnector, Body};
use std::net::SocketAddr;

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

    let uri = format!("http://100.91.35.113:32400{}", path_query);
    dbg!("PROMOTED");
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
