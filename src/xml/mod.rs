use async_trait::async_trait;
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
use axum_core::extract::FromRequest;
use axum_core::BoxError;
use http::header::{self, HeaderMap, HeaderValue};
use http::StatusCode;
use hyper::Body;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::to_writer;
use yaserde;
use yaserde::de::from_str as from_xml_str;
use yaserde::ser::to_string as to_xml_str;
use yaserde::YaSerialize;
use yaserde::YaDeserialize;
// use axum::extract::rejection::XmlRejection;
// use axum::extract::rejection::FormRejection;
// use axum_core::extract::rejection::BytesRejection;
// use axum_core::extract::rejection::*;
use bytes::Bytes;
use self::rejection::XmlRejection;

pub mod rejection;

#[derive(Debug, Clone, Copy, Default)]
pub struct Xml<T: YaDeserialize + YaSerialize>(pub T);

#[async_trait]
impl<T: YaDeserialize + YaSerialize, S, B> FromRequest<S, B> for Xml<T>
where
    T: DeserializeOwned,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = XmlRejection;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        if xml_content_type(req.headers()) {
            let bytes = Bytes::from_request(req, state).await?;
            let value = yaserde::de::from_reader(&*bytes).unwrap();

            Ok(Self(value))
        } else {
            Err(XmlRejection::MissingXMLContentType)
        }
    }
}

fn xml_content_type(headers: &HeaderMap) -> bool {
    let content_type = if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        content_type
    } else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return false;
    };

    let mime = if let Ok(mime) = content_type.parse::<mime::Mime>() {
        mime
    } else {
        return false;
    };

    let is_xml_content_type = (mime.type_() == "application" || mime.type_() == "text")
        && (mime.subtype() == "xml"
            || mime.suffix().map_or(false, |name| name == "xml"));

    is_xml_content_type
}

impl<T> IntoResponse for Xml<T>
where
    T: YaDeserialize + YaSerialize,
{
    fn into_response(self) -> Response {
        // let mut bytes = Vec::new();
        match to_xml_str(&self.0) {
            Ok(v) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::XML.as_ref()),
                )],
                v,
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}

// define_rejection! {
//     #[status = UNSUPPORTED_MEDIA_TYPE]
//     #[body = "Expected request with `Content-Type: text/xml`"]
//     // #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
//     /// Rejection type for [`Json`](super::Json) used if the `Content-Type`
//     /// header is missing.
//     pub struct MissingXmlContentType;
// }

// #[derive(Debug)]
// pub enum XmlRejection {
//     // #[error("Expected request with `Content-Type: application/xml`")]
//     MissingXMLContentType,
// }

// impl IntoResponse for XmlRejection {
//     fn into_response(self) -> Response {
//         match self {
//             e @ XmlRejection::MissingXMLContentType => {
//                 let mut res = Response::new(body::boxed(Full::from(format!("{}", e))));
//                 *res.status_mut() = StatusCode::UNSUPPORTED_MEDIA_TYPE;
//                 res
//             }
//         }
//     }
// }
