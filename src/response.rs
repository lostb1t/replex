use std::io::Cursor;
use std::io::Write;
use std::str;
// use YaSerialize;
use bytes::Bytes;
use bytes::BytesMut;
use salvo::test::ResponseExt;
use serde::Serialize;
use yaserde::YaSerialize;
use yaserde::YaDeserialize;
use yaserde::ser::Config;
use yaserde::ser::Serializer;
// use yaserde::ser::serialize_with_writer;
use yaserde::ser::to_string as to_xml_str;
use async_trait::async_trait;
use yaserde;
use salvo::Piece;
use salvo::http::header::{HeaderValue, CONTENT_TYPE};
use salvo::http::{Response, StatusError};
use salvo::writing::Json;


use crate::models::MediaContainer;
use crate::models::MediaContainerWrapper;
use crate::utils::*;


impl<T> Piece for MediaContainerWrapper<T>
    where
    T: Serialize  + YaSerialize + Send,
{
    #[inline]
    fn render(self, res: &mut Response) {
        match &self.content_type {
            ContentType::Json => Json(self).render(res),
            ContentType::Xml => Json(self).render(res),
        }
    }
}

pub struct Xml<T>(pub T);



#[async_trait]
impl<T> Piece for Xml<T>
where
    T: YaSerialize + Send
{
    #[inline]
    fn render(self, res: &mut Response) {
        // let bytes = res.take_body();
        // let bytes = res.take_bytes(Some(&mime::TEXT_XML)).await.unwrap();
        // let b = res.to_string();
        // let value = to_xml_str(&self.0);
        match to_xml_str(&self.0) {
            Ok(bytes) => {
                res.headers_mut().insert(
                    CONTENT_TYPE,
                    HeaderValue::from_static("text/xml; charset=utf-8"),
                );
                res.write_body(bytes).ok();
            }
            Err(e) => {
                tracing::error!(error = ?e, "Xml write error");
                res.render(StatusError::internal_server_error());
            }
        }
    }
}