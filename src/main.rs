#[macro_use]
extern crate tracing;

use axum::{
    body::Body,
    extract::Path,
    extract::State,
    response::Redirect,
    // http::{uri::Uri, Request, Response},
    routing::get,
    Router,
};
use std::{convert::Infallible, env, net::SocketAddr, time::Duration};
// use axum::headers::ContentType;

// use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
// use axum_tracing_opentelemetry::middleware::OtelInResponseLayer;
use http::{Request, Response};

// use hyper::{client::HttpConnector, Body};

// use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
use itertools::Itertools;
use replex::models::*;
use replex::plex_client::*;
use replex::proxy::*;
use replex::routes::*;
use replex::config::*;
use replex::url::*;
use replex::utils::*;
use tower::ServiceBuilder;
use tower_http::cors::AllowOrigin;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::Registry;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // set_default_env_var("REPLEX_", "8080");
    // let new_relic_api_key = SETTINGS.read().unwrap().get::<String>("host");
    // env_logger::init();
    // https://github.com/tokio-rs/axum/blob/main/examples/tracing-aka-logging/src/main.rs
    // if let new_relic_api_key = SETTINGS.read().unwrap().get::<String>("newrelic_api_key").unwrap() {
    //     let newrelic = tracing_newrelic::layer(new_relic_api_key);
    //     tracing_subscriber::registry()
    //         .with(newrelic)
    //         .with(tracing_subscriber::fmt::layer())
    //         .init();

    //     // let fmt = tracing_subscriber::fmt::layer();
    //     // let subscriber = Registry::default().with(newrelic).with(fmt).with(target);
    //     // tracing::subscriber::set_global_default(subscriber)
    //     //     .expect("failed to initilize tracing subscriber");
    // } else {
    //     tracing_subscriber::fmt::init();
    // }

    // let content_type = if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
    //     content_type
    // } else {
    //     return false;
    // };

    tracing_subscriber::fmt::init();
    // env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "https://otlp.eu01.nr-data.net");
    //OTEL_EXPORTER_OTLP_TRACES_ENDPOINT
    // init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers().unwrap();

    let proxy = Proxy::default();
    let addr = SocketAddr::from(([0, 0, 0, 0], 80));
    info!(message = "Listening on", %addr);
    axum::Server::bind(&addr)
        .serve(router(proxy).into_make_service())
        .await
        .unwrap();
}
