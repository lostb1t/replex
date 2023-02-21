use crate::models::*;
use crate::utils::*;
use anyhow::Result;
use axum::http::{uri::Uri, Request};
// use crate::axum::http::{uri::Uri, Request, Response};
use crate::models::*;
use crate::settings::*;
use cached::proc_macro::cached;
use http::HeaderValue;
use hyper::Body;

use std::convert::TryFrom;

// struct MyRequest<T>(Request<T>);
// // pub struct Request<T> {
// //     head: Parts,
// //     body: T,
// // }

// impl<T: fmt::Debug> fmt::Debug for Request<T> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         self.0
//     }
// }

#[derive(Debug)]
pub struct Proxy {
    pub client: hyper::client::Client<hyper::client::HttpConnector, Body>,
    pub host: String,
    // pub headers:
    // pub plex_api: Option<plex_api::Server>,
    // pub req: MyRequest<Body>,
}

impl Proxy {
    // pub fn set_x_plex_token(self, token: String) {

    // }
    // pub fn clone_for_req(&self) -> Self {
    //     Proxy {
    //         client: self.client.clone(),
    //         host: self.host.clone(),
    //         plex_api: self.plex_api.clone()
    //     }
    // }

    // pub async fn set_plex_api_from_request(&mut self, req: &Request<Body>) -> &mut Self {
    //     let plex_client = create_client_from_request(req).unwrap();
    //     let plex_api = plex_api::Server::new("http://100.91.35.113:32400", plex_client)
    //         .await
    //         .unwrap();

    //     // self.req = request;
    //     self.plex_api = Some(plex_api);
    //     self
    // }

    // pub async fn set_headers(&mut self, req: Request<Body>) -> &mut Self {
    //     let plex_client = create_client_from_request(&req).unwrap();
    //     let plex_api = plex_api::Server::new("http://100.91.35.113:32400", plex_client)
    //         .await
    //         .unwrap();

    //     let (mut parts, _) = req.into_parts();
    //     let mut request = Request::from_parts(parts, Body::empty());

    //     self.req = request;
    //     self.plex_api = plex_api;
    //     self
    // }

    // pub fn get_x_plex_token(&self) -> String {
    //     // dbg!(&self.plex_api);
    //     self.plex_api.as_ref().unwrap().client().x_plex_token().to_string()
    // }

    // pub fn proxy(&self) -> hyper::client::ResponseFuture {
    //     self.request(self.req)
    // }

    pub fn request(&self, mut req: Request<Body>) -> hyper::client::ResponseFuture {
        let path = req.uri().path();
        let path_query = req
            .uri()
            .path_and_query()
            .map(|v| v.as_str())
            .unwrap_or(path);
        let uri = format!("{}{}", self.host, path_query);
        // dbg!(&uri);
        // Default is gzip. Dont want that
        req.headers_mut()
            .insert("Accept-Encoding", HeaderValue::from_static("identity"));

        dbg!(&uri);
        *req.uri_mut() = Uri::try_from(uri).unwrap();
        self.client.request(req)
    }

    // async fn get_collections(&self) -> Result<Vec<MetaData>> {
    //     // let plex_client = create_client_from_request(&req).unwrap();
    //     // let plex_api = plex_api::Server::new("http://100.91.35.113:32400", plex_client).await.unwrap();
    //     let mut collections = vec![];
    //     let api = self.plex_api.clone().unwrap();
    //     for library in api.libraries() {
    //         // library.media

    //         let mut resp: MediaContainerWrapper<MediaContainer> = api
    //             .client()
    //             .get(format!("/library/sections/{}/collections", library.id()))
    //             .json()
    //             .await?;
    //         collections.append(&mut resp.media_container.metadata);
    //     }
    //     // println!("no cache");
    //     Ok(collections)
    // }

    // pub async fn get_promoted_hubs(
    //     &self,
    //     mut req: Request<Body>,
    // ) -> Result<MediaContainerWrapper<MediaContainer>> {
    //     let uri = format!("{}{}", self.host, "/hubs/promoted");
    //     *req.uri_mut() = Uri::try_from(uri).unwrap();
    //     let mut resp = self.client.request(req).await?;
    //     // trace!("Got {:#?}", resp);
    //     // from_response(resp).await
    //     // debug!("Getting promoted hubs");
    //     // let req = remove_param(req, "contentDirectoryID".to_owned()).await;
    //     // // req.headers_mut().remove("contentDirectoryID");
    //     // trace!("Proxy call {:#?}", req);
    //     // let mut resp = PROXY_CLIENT
    //     //     .call(client_ip, "http://100.91.35.113:32400", req)
    //     //     .await
    //     //     .unwrap();
    //     // trace!("Got {:#?}", resp);
    //     // from_response(resp).await
    // }
}

impl Default for Proxy {
    fn default() -> Self {
        Self {
            host: SETTINGS.read().unwrap().get::<String>("host").unwrap(),
            client: HttpClient::new(),
        }
    }
}

impl Clone for Proxy {
    fn clone(&self) -> Proxy {
        // let (mut parts, _) = self.req.into_parts();
        // let mut request = Request::from_parts(parts, Body::empty());

        Proxy {
            client: self.client.clone(),
            host: self.host.clone(),
            // plex_api: self.plex_api.clone(),
            // req: self.req.clone(),
        }
    }
}

// #[cached(
//     time = 720,
//     key = "String",
//     convert = r#"{ proxy.get_x_plex_token() }"#
// )]
// pub async fn get_cached_collections(proxy: &Proxy) -> Vec<MetaData> {
//     proxy.get_collections().await.unwrap()
// }
// pub async fn get_cached_collections(proxy: &Proxy) -> Vec<MetaData> {
//     proxy.get_collections().await
// }

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
