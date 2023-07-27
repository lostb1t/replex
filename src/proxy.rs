use crate::{models::*, config::Config};
use salvo::BoxedError;
// use salvo::extract;
use salvo::http::header::HeaderValue;
use salvo::macros::Extractible;
use salvo::prelude::*;
use salvo::proxy::Proxy as SalvoProxy;
use salvo::proxy::Upstreams;
use salvo::{
    http::response::Response as SalvoResponse, test::ResponseExt,
    Extractible, Request as SalvoRequest,
};
use tracing::{debug, instrument};


use std::convert::TryFrom;


pub struct PlexProxy<U> {
    pub proxy: SalvoProxy<U>,
}

impl<U> PlexProxy<U>
where
    U: Upstreams,
    U::Error: Into<BoxedError>,
{
    pub fn new(upstreams: U) -> Self {
        PlexProxy {
            proxy: SalvoProxy::new(upstreams),
        }
    }

    pub async fn request(&self, req: &mut SalvoRequest) -> Response {
        let mut depot = Depot::with_capacity(1);
        let mut res = Response::new();
        let mut ctrl = FlowCtrl::new(vec![]);
        // disable gzip
        req.headers_mut()
            .insert("Accept-Encoding", HeaderValue::from_static("identity"));
        debug!("Making request: {:?}", &req);
        self.handle(req, &mut depot, &mut res, &mut ctrl).await;
        res
    }
}

#[async_trait]
impl<U> Handler for PlexProxy<U>
where
    U: Upstreams,
    U::Error: Into<BoxedError>,
{
    #[inline]
    async fn handle(
        &self,
        req: &mut SalvoRequest,
        _depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        // self.proxy.build_proxied_request(req);
        self.proxy.handle(req, _depot, res, ctrl).await
    }
}
