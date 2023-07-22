use axum::{
    http::{Request},
    response::{IntoResponse, Response},
    Json,
    body::Body,
};

use serde::Serialize;
use yaserde::YaSerialize;
use yaserde::YaDeserialize;

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