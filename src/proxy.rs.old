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

use crate::config::Config;

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
        mut req: &mut salvo::Request,
        depot: &mut Depot,
        res: &mut salvo::Response,
        ctrl: &mut FlowCtrl,
    ) {
        // req.he
        //bit of a hack
        // let config: Config = Config::dynamic(req).extract().unwrap();
        // req = req.add_header("HOST", config.host.unwrap(), true).unwrap();
        // self.ups
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
