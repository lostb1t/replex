extern crate tracing;
use anyhow::Result;
use axum::{
    body::HttpBody,
    extract::State,
    http::{uri::Uri, Request, Response},
    routing::get,
    Router,
};
use hyper::{client::HttpConnector, Body};

pub fn remove_param(mut req: Request<Body>, param: &str) -> Request<Body> {
    let mut uri = pathetic::Uri::default()
        .with_path(req.uri_mut().path())
        .with_query(req.uri_mut().query());
    let query: Vec<(String, String)> = uri
        .query_pairs()
        .filter(|(name, _)| name != param)
        .map(|(name, value)| (name.into_owned(), value.into_owned()))
        .collect();
    uri.query_pairs_mut().clear().extend_pairs(query);
    *req.uri_mut() = Uri::try_from(uri.as_str()).unwrap();
    req
}