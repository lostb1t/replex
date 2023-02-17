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
use axum_core::body;
use axum_core::extract::rejection::BytesRejection;
use http::StatusCode;
use thiserror::Error;

// use self::Xml;

#[derive(Debug, Error)]
pub enum XmlRejection {
    #[error("Expected request with `Content-Type: application/xml`")]
    MissingXMLContentType,
    #[error("{0}")]
    BytesRejection(#[from] BytesRejection),
}

impl axum::response::IntoResponse for XmlRejection {
    fn into_response(self) -> Response {
        // let code = match self {
        //     ApiError::JsonExtractorRejection(x) => match x {
        //         JsonRejection::JsonDataError(_) => StatusCode::UNPROCESSABLE_ENTITY,
        //         JsonRejection::JsonSyntaxError(_) => StatusCode::BAD_REQUEST,
        //         JsonRejection::MissingJsonContentType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        //         _ => StatusCode::INTERNAL_SERVER_ERROR,
        //     },
        // };
        let code = match self {
            // e @ XmlRejection::InvalidXMLBody(_) => StatusCode::UNPROCESSABLE_ENTITY,
            e @ XmlRejection::MissingXMLContentType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            XmlRejection::BytesRejection(e) => StatusCode::UNPROCESSABLE_ENTITY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (code).into_response()
        // match self {
        //     e @ XmlRejection::InvalidXMLBody(_) => {
        //         let mut res = Response::new(body::boxed(Full::from(format!("{}", e))));
        //         *res.status_mut() = StatusCode::UNPROCESSABLE_ENTITY;
        //         res
        //     }
        //     e @ XmlRejection::MissingXMLContentType => {
        //         let mut res = Response::new(body::boxed(Full::from(format!("{}", e))));
        //         *res.status_mut() = StatusCode::UNSUPPORTED_MEDIA_TYPE;
        //         res
        //     }
        //     XmlRejection::BytesRejection(e) => e.into_response(),
        // }
    }
}