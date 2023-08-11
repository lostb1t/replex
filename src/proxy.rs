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
use indexmap::IndexMap;
use salvo::http::header::HeaderValue;

// use reqwest::Client;
use salvo::http::header::CONNECTION;
use salvo::http::header::UPGRADE;
// use salvo::proxy::Proxy as SalvoProxy;
use salvo::proxy::Proxy as SalvoProxy;
use salvo::proxy::Upstreams;
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
                .url_rest_getter(default_url_rest_getter),
        }
    }
    /// Create new `Proxy` with upstreams list and [`Client`].
    pub fn with_client(upstreams: U, client: reqwest::Client) -> Self {
        Proxy {
            inner: SalvoProxy::with_client(upstreams, client)
                .url_rest_getter(default_url_rest_getter),
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
            ).url_rest_getter(default_url_rest_getter),
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
        // dbg!(&req.uri_mut());
        // let mut reqq = RequestBuilder::new(req.uri_mut().to_string(), req.method_mut().clone()).build();
        self.inner.handle(req, depot, res, ctrl).await;
    }
}

pub fn default_url_rest_getter(req: &Request, _depot: &Depot) -> String {
    req.uri().path_and_query().unwrap().to_string()
}
