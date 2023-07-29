

use serde::Serialize;
use yaserde::YaSerialize;
use yaserde::ser::to_string as to_xml_str;
use async_trait::async_trait;
use yaserde;
use salvo::Piece;
use salvo::http::header::{HeaderValue, CONTENT_TYPE};
use salvo::http::{Response, StatusError};
use salvo::writing::Json;
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
            ContentType::Xml => Xml(self.media_container).render(res),
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
    fn render(self, res: &mut Response) {;
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