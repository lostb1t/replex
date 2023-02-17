use axum::{
    body::HttpBody,
    extract::State,
    http::{uri::Uri, Request},
    response::{IntoResponse, Response},
    // response::{IntoResponse, Response},
    routing::get,
    Json,
    Router,
};
use async_trait::async_trait;
use axum_core::extract::{FromRequest};
use http::header::{self, HeaderMap, HeaderValue};
use http::{StatusCode};
use hyper::Body;
use serde::Serialize;
use serde_json::to_writer;
use yaserde::de::from_str as from_xml_str;
use yaserde::ser::to_string as to_xml_str;
use yaserde::YaSerialize;
use yaserde::YaDeserialize;
use serde::de::DeserializeOwned;
use crate::utils::*;
use crate::xml::*;

// #[derive(Debug, Clone)]
pub struct PlexResponse<T: Serialize + YaSerialize> {
    pub body: T,
    pub req: Request<Body>,
}

impl<T> IntoResponse for PlexResponse<T>
where
    T: Serialize + YaDeserialize + YaSerialize,
{
    fn into_response(self) -> Response {
        let content_type = get_content_type(self.req);
        match content_type {
            ContentType::Json => Json(self.body).into_response(),
            ContentType::Xml => Xml(self.body).into_response(),
        }
    }
}