use anyhow::Result;
use axum::{
    body::HttpBody,
    extract::State,
    http::{uri::Uri, Request, Response},
    routing::get,
    Router,
};
use hyper::client::connect::Connect;
use hyper::{client::HttpConnector, Body};
use std::error::Error as StdError;
use std::{error::Error, net::SocketAddr};


#[derive(Debug)]
pub struct Proxy {
    pub client: hyper::client::Client<hyper::client::HttpConnector, Body>,
    pub host: String,
}

impl Proxy
{
    pub fn request(&self, mut req: Request<Body>) -> hyper::client::ResponseFuture {
        let path = req.uri().path();
        let path_query = req
            .uri()
            .path_and_query()
            .map(|v| v.as_str())
            .unwrap_or(path);
        let uri = format!("{}{}", self.host, path_query);
        // dbg!(&uri);
        *req.uri_mut() = Uri::try_from(uri).unwrap();
        self.client.request(req)
        // dbg!("yup").to_string()
    }
}

impl Clone for  Proxy {
    fn clone(&self) -> Proxy {
        Proxy {
            client: self.client.clone(),
            host: self.host.clone(),
        }
    }
}

// #[derive(Debug)]
// pub struct ProxyClient<C, B = Body> {
//     pub client: hyper::client::Client<C, B>,
//     pub host: String,
// }

// impl<C, B> ProxyClient<C, B>
// where
//     C: Connect + Clone + Send + Sync + 'static,
//     B: HttpBody + Send + 'static,
//     B::Data: Send,
//     B::Error: Into<Box<dyn StdError + Send + Sync>>,
// {
//     pub fn proxy_request(&self, mut req: Request<B>) -> hyper::client::ResponseFuture {
//         let path = req.uri().path();
//         let path_query = req
//             .uri()
//             .path_and_query()
//             .map(|v| v.as_str())
//             .unwrap_or(path);
//         let uri = format!("{}{}", self.host, path_query);
//         // dbg!(&uri);
//         *req.uri_mut() = Uri::try_from(uri).unwrap();
//         self.client.request(req)
//         // dbg!("yup").to_string()
//     }
// }

// impl<C: Clone, B> Clone for ProxyClient<C, B> {
//     fn clone(&self) -> ProxyClient<C, B> {
//         ProxyClient {
//             client: self.client.clone(),
//             host: self.host.clone(),
//         }
//     }
// }

// impl<C, B> ProxyRequest for hyper::client::Client<C, B> {
//     fn proxy_request(&self, mut req: Request<B>) -> hyper::client::ResponseFuture {
//         let path = req.uri().path();
//         let path_query = req
//             .uri()
//             .path_and_query()
//             .map(|v| v.as_str())
//             .unwrap_or(path);
//         let uri = format!("http://100.91.35.113:32400{}", path_query);
//         dbg!(&uri);
//         *req.uri_mut() = Uri::try_from(uri).unwrap();
//         self.request(req).await.unwrap();
//         // dbg!("yup").to_string()
//     }
// }

// impl Summary for NewsArticle {
// impl<C, B> hyper::client::Client<C, B>
// where
//     C: Connect + Clone + Send + Sync + 'static,
//     B: HttpBody + Send + 'static,
//     B::Data: Send,
//     B::Error: Into<Box<dyn StdError + Send + Sync>>,
// {
//     fn summarize(&self) -> String {
//         format!("{}, by {} ({})", self.headline, self.author, self.location)
//     }
// }