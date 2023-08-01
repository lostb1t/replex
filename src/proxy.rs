use hyper::HeaderMap;
// use hyper::header::CONNECTION;
use salvo::http::ReqBody;
use salvo::http::ResBody;
use salvo::BoxedError;
use salvo::Error;
// use salvo::extract;
use salvo::http::header::HeaderValue;

// use reqwest::Client;
use salvo::http::header::CONNECTION;
use salvo::http::header::UPGRADE;
use salvo::prelude::*;
use salvo::proxy::Proxy as SalvoProxy;
use salvo::proxy::Upstreams;
use salvo::rt::TokioIo;
use tokio::io::copy_bidirectional;
use tracing::debug;

type HyperRequest = hyper::Request<ReqBody>;
type HyperResponse = hyper::Response<ResBody>;

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

    pub async fn request(&self, req: &mut Request) -> Response {
        let mut depot = Depot::with_capacity(1);
        let mut res = Response::new();
        let mut ctrl = FlowCtrl::new(vec![]);
        // disable gzip
        req.headers_mut()
            .insert("Accept-Encoding", HeaderValue::from_static("identity"));
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
        req: &mut Request,
        _depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        // self.websocket(req, res).await
        self.proxy.handle(req, _depot, res, ctrl).await
    }
}
