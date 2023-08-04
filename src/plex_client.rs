use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::models::*;
use crate::utils::*;
use anyhow::Result;

use futures_util::Future;
use futures_util::TryStreamExt;
// use hyper::client::HttpConnector;
// use hyper::Body;
use hyper::body::Body;
use moka::future::Cache;
use moka::future::ConcurrentCacheExt;
use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use reqwest::header;
use reqwest::header::HeaderValue;
use reqwest::Client;
use salvo::http::ReqBody;
use salvo::Error;
use salvo::Request;
use salvo::Response;
// use hyper::client::HttpConnector;

use salvo::http::ResBody;

static CACHE: Lazy<Cache<String, MediaContainerWrapper<MediaContainer>>> =
    Lazy::new(|| {
        let c: Config = Config::figment().extract().unwrap();
        Cache::builder()
            .max_capacity(10000)
            .time_to_live(Duration::from_secs(c.cache_ttl))
            .build()
    });

#[derive(Debug, Clone)]
pub struct PlexClient {
    pub http_client: Client,
    pub host: String, // TODO: Dont think this suppsoed to be here. Should be higher up
    pub cache: Cache<String, MediaContainerWrapper<MediaContainer>>,

    // /// `X-Plex-Platform` header value.
    // ///
    // /// Platform name, e.g. iOS, macOS, etc.
    pub x_plex_platform: Option<String>,

    // /// `X-Plex-Device-Name` header value.
    // ///
    // /// Primary name for the device, e.g. "Plex Web (Chrome)".
    // pub x_plex_device_name: String,
    /// `X-Plex-Client-Identifier` header value.
    ///
    /// UUID, serial number, or other number unique per device.
    ///
    /// **N.B.** Should be unique for each of your devices.
    pub x_plex_client_identifier: String,

    /// `X-Plex-Token` header value.
    ///
    /// Auth token for Plex.
    pub x_plex_token: String,
}

impl PlexClient {
    // TODO: Handle 404s/500 etc
    // TODO: Map reqwest response and error to salvo
    pub async fn get(&self, path: String) -> Result<reqwest::Response, Error> {
        let uri = format!("{}{}", self.host, path);
        let res = self
            .http_client
            .get(uri)
            .send()
            .await
            .map_err(Error::other)?;
        Ok(res)
    }

    pub async fn request(
        &self,
        req: &mut Request,
    ) -> Result<reqwest::Response, Error> {
        let uri = format!(
            "{}{}",
            self.host,
            &req.uri_mut().path_and_query().unwrap()
        );
        // let l = &req.uri_mut().path_and_query();
        // dbg!(&self.host);
        // dbg!(&req.));
        let mut headers = req.headers_mut().to_owned();
        let res = self
            .http_client
            .get(uri)
            .headers(headers)
            .send()
            .await
            .map_err(Error::other)?;
        Ok(res)
    }

    // pub fn request(&self, req) -> hyper::client::ResponseFuture {
    //     self.http_client.request(req)
    // }

    pub async fn get_section_collections(
        &self,
        id: u32,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let res = self
            .get(format!("/library/sections/{}/collections", id))
            .await
            .unwrap();

        let mut container: MediaContainerWrapper<MediaContainer> =
            from_reqwest_response(res)
                .await
                .expect("Cannot get MediaContainerWrapper from response");

        Ok(container)
    }

    pub async fn get_collection_children(
        &self,
        id: u32,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let mut path = format!("/library/collections/{}/children", id);

        if offset.is_some() {
            path =
                format!("{}?X-Plex-Container-Start={}", path, offset.unwrap());
        }

        if limit.is_some() {
            path = format!("{}&X-Plex-Container-Size={}", path, limit.unwrap());
        }
        let resp = self.get(path).await.unwrap();
        let container: MediaContainerWrapper<MediaContainer> =
            from_reqwest_response(resp).await.unwrap();
        Ok(container)
    }

    pub async fn get_collection(
        &self,
        id: i32,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let resp = self
            .get(format!("/library/collections/{}", id))
            .await
            .unwrap();
        let container: MediaContainerWrapper<MediaContainer> =
            from_reqwest_response(resp).await.unwrap();
        Ok(container)
    }

    pub async fn get_item_by_key(
        self,
        key: String,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let resp = self.get(key).await.unwrap();
        let container: MediaContainerWrapper<MediaContainer> =
            from_reqwest_response(resp).await.unwrap();
        Ok(container)
    }

    pub async fn get_cached(
        self,
        f: impl Future<Output = Result<MediaContainerWrapper<MediaContainer>>>,
        name: String,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let cache_key = self.generate_cache_key(name.clone());
        let cached = self.get_cache(&cache_key).await.unwrap();

        if cached.is_some() {
            return Ok(cached.unwrap());
        }
        let r = f.await.unwrap();
        self.insert_cache(cache_key, r.clone()).await;
        Ok(r)
    }

    async fn get_cache(
        &self,
        cache_key: &str,
    ) -> Result<Option<MediaContainerWrapper<MediaContainer>>> {
        Ok(self.cache.get(cache_key))
    }

    async fn insert_cache(
        &self,
        cache_key: String,
        container: MediaContainerWrapper<MediaContainer>,
    ) {
        self.cache.insert(cache_key, container).await;
        self.cache.sync();
    }

    fn generate_cache_key(&self, name: String) -> String {
        format!("{}:{}", name, self.x_plex_token)
    }
}

impl PlexClient {
    pub fn from_request(req: &Request, params: PlexParams) -> Self {
        let config: Config = Config::figment().extract().unwrap();
        let token = params
            .clone()
            .token
            .expect("Expected to have an token in header or query");
        let client_identifier = params
            .clone()
            .client_identifier
            .expect("Expected to have an plex client identifier header");
        let platform = params.clone().platform;

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "X-Plex-Token",
            header::HeaderValue::from_str(token.clone().as_str()).unwrap(),
        );
        headers.insert(
            "X-Plex-Client-Identifier",
            header::HeaderValue::from_str(client_identifier.clone().as_str())
                .unwrap(),
        );
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/json"),
        );
        if let Some(i) = platform.clone() {
            headers.insert(
                "X-Plex-Platform",
                header::HeaderValue::from_str(i.as_str()).unwrap(),
            );
        }
        Self {
            http_client: reqwest::Client::builder()
                .default_headers(headers)
                .gzip(true)
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            host: config.host.unwrap(),
            x_plex_token: token,
            x_plex_client_identifier: client_identifier,
            x_plex_platform: platform,
            cache: CACHE.clone(),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use salvo::prelude::*;
//     use salvo::test::{ResponseExt, TestClient};
//     use crate::test_helpers::*;

//     #[tokio::test]
//     async fn test_hello_world() {
//         let service = Service::new(super::route());

//         let content = TestClient::get(format!("http://127.0.0.1:5800/{}", "hubs/promoted"))
//             .send((&service))
//             .await
//             .take_string()
//             .await
//             .unwrap();
//         assert_eq!(content, "Hello World");
//     }
// }
