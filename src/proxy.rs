use hyper::http;
use hyper::HeaderMap;
// use hyper::header::CONNECTION;
use once_cell::sync::OnceCell;
use salvo::http::ReqBody;
use salvo::http::ResBody;
use salvo::BoxedError;
use salvo::Error;
// use salvo::extract;
use http::uri::{Scheme, Uri};
use http::Extensions;
use salvo::http::header::HeaderValue;
use std::fmt;
// use reqwest::Client;
use salvo::http::header::CONNECTION;
use salvo::http::header::UPGRADE;
// use salvo::proxy::Proxy as SalvoProxy;
use salvo::proxy::Proxy as SalvoProxy;
use salvo::proxy::Upstreams;
// salvo_core::handler::Handler;
use salvo::rt::TokioIo;
use salvo::test::ResponseExt;
// use std::net::SocketAddr;
use tokio::io::copy_bidirectional;
use tracing::debug;
use url::Url;

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::client::conn::http1::Builder;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper::Method;
use salvo::test::RequestBuilder;
use salvo::{
    async_trait, conn::SocketAddr, http::Version, Depot, FlowCtrl, Handler,
    Request, Response,
};
use tokio::net::{TcpListener, TcpStream};

type HyperRequest = hyper::Request<ReqBody>;
type HyperResponse = hyper::Response<ResBody>;


pub struct Proxy {
    pub inner: SalvoProxy<String>,
}

impl fmt::Debug for Proxy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Proxy")
    }
}

impl Proxy {
// impl<U> Proxy<U>
// where
//     U: Upstreams,
//     U::Error: Into<BoxedError>,
// {
    /// Create new `Proxy` with upstreams list.
    pub fn new(upstream: String) -> Self {
        Proxy {
            inner: SalvoProxy::new(upstream)
                .url_path_getter(default_url_path_getter)
                .url_query_getter(default_url_query_getter),
        }
    }
    /// Create new `Proxy` with upstreams list and [`Client`].
    pub fn with_client(upstream: String, client: reqwest::Client) -> Self {
        Proxy {
            inner: SalvoProxy::with_client(upstream, client)
                .url_path_getter(default_url_path_getter)
                .url_query_getter(default_url_query_getter),
        }
    }

    #[inline]
    pub async fn request(
        &self,
        req: &mut salvo::Request,
    ) -> Result<salvo::Response, BoxedError> {
        let mut depot = Depot::new();
        let mut res = Response::new();
        let mut ctrl = FlowCtrl::new(vec![]);
        self.inner.handle(req, &mut depot, &mut res, &mut ctrl).await;
        Ok(res)
    }
}


impl Clone for Proxy {
// impl<U> Clone for Proxy<U>
// where
//     U: Upstreams + Clone,
//     U::Error: Into<BoxedError>,
// {
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
impl Handler for Proxy {
// impl<U> Handler for Proxy<U>
// where
//     U: Upstreams,
//     U::Error: Into<BoxedError>,
// {
    #[inline]
    async fn handle(
        &self,
        req: &mut salvo::Request,
        depot: &mut Depot,
        res: &mut salvo::Response,
        ctrl: &mut FlowCtrl,
    ) {
        // dbg!(&req.uri_mut());
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
