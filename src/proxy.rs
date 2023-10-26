use hyper::http;
use salvo::http::ReqBody;
use salvo::http::ResBody;
use salvo::BoxedError;
use salvo::proxy::Proxy as SalvoProxy;
use salvo::proxy::Upstreams;

use salvo::{
    async_trait, conn::SocketAddr, http::Version, Depot, FlowCtrl, Handler,
    Request, Response,
};
use tokio::net::{TcpListener, TcpStream};

type HyperRequest = hyper::Request<ReqBody>;
type HyperResponse = hyper::Response<ResBody>;

pub struct Proxy<U> {
    pub inner: SalvoProxy<U>,
}

impl<U> Proxy<U>
where
    U: Upstreams,
    U::Error: Into<BoxedError>,
{
    /// Create new `Proxy` with upstreams list.
    pub fn new(upstreams: U) -> Self {
        Proxy {
            inner: SalvoProxy::new(upstreams)
                .url_path_getter(default_url_path_getter)
                .url_query_getter(default_url_query_getter),
        }
    }
    /// Create new `Proxy` with upstreams list and [`Client`].
    pub fn with_client(upstreams: U, client: reqwest::Client) -> Self {
        Proxy {
            inner: SalvoProxy::with_client(upstreams, client)
                .url_path_getter(default_url_path_getter)
                .url_query_getter(default_url_query_getter),
        }
    }
}

impl<U> Clone for Proxy<U>
where
    U: Upstreams + Clone,
    U::Error: Into<BoxedError>,
{
    fn clone(&self) -> Self {
        let upstreams = self.inner.upstreams().clone();
        Proxy {
            inner: SalvoProxy::with_client(
                upstreams,
                self.inner.client.clone(),
            )
            .url_path_getter(default_url_path_getter)
            .url_query_getter(default_url_query_getter),
        }
    }

    // fn clone_from(&mut self, source: &Self) {
    //     *self = source.clone()
    // }
}

#[async_trait]
impl<U> Handler for Proxy<U>
where
    U: Upstreams,
    U::Error: Into<BoxedError>,
{
    #[inline]
    async fn handle(
        &self,
        req: &mut salvo::Request,
        depot: &mut Depot,
        res: &mut salvo::Response,
        ctrl: &mut FlowCtrl,
    ) {
        // let mut reqq = RequestBuilder::new(req.uri_mut().to_string(), req.method_mut().clone()).build();
        self.inner.handle(req, depot, res, ctrl).await;
    }
}

pub fn default_url_path_getter(
    req: &Request,
    _depot: &Depot,
) -> Option<String> {
    Some(req.uri().path().to_string())
}

pub fn default_url_query_getter(
    req: &Request,
    _depot: &Depot,
) -> Option<String> {
    // dbg!(&req.uri().query());
    match req.uri().query() {
        Some(i) => Some(i.to_string()),
        _ => None
    }
}
