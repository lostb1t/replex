use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::models::*;
use crate::proxy::PlexProxy;
use crate::utils::*;
use anyhow::Result;

use futures_util::Future;
use futures_util::TryStreamExt;
// use hyper::client::HttpConnector;
// use hyper::Body;
use hyper::body::Body;
use moka::future::Cache;
use moka::future::ConcurrentCacheExt;
use once_cell::sync::OnceCell;
use reqwest::header;
use reqwest::Client;
use salvo::http::ReqBody;
use salvo::Error;
use salvo::Request;
use salvo::Response;
use once_cell::sync::Lazy;
// use hyper::client::HttpConnector;

use salvo::http::ResBody;

// type HttpClient = hyper::client::Client<HttpConnector, Body>;
// pub static MOKA_CACHE: RwLock<MokaCache<String, Arc<Vec<u8>>>> = RwLock::new(MokaCache::new(250));
// static CACHE: OnceCell<Cache<String, MediaContainerWrapper<MediaContainer>>> =
//     OnceCell::new(
//         Cache::builder()
//             .max_capacity(10000)
//             .time_to_live(Duration::from_secs(Config::figment().extract().unwrap().cache_ttl))
//             .build()
//     );

static CACHE: Lazy<Cache<String, MediaContainerWrapper<MediaContainer>>> = Lazy::new(|| {
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
    pub x_plex_platform: String,

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

    // TODO: Should return a reqw response
    pub async fn request(&self, req: &mut Request) -> Response {
        let path = req.uri().path();
        let upstream = format!("{}{}", self.host.clone(), path);
        let proxy = PlexProxy::new(upstream);
        proxy.request(req).await
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
            dbg!(cache_key.clone());
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
    pub fn new(req: &mut Request, params: PlexParams) -> Self {
        // TODO: Split it into a function from_request
        // TODO: Dont need request
        let config: Config = Config::figment().extract().unwrap();
        let token = params
            .clone()
            .token
            .expect("Expected to have an token in header or query");
        let client_identifier = params
            .clone()
            .client_identifier
            .expect("Expected to have an plex client identifier header");
        let platform = params
            .clone()
            .platform
            .expect("Expected to have an plex platform header");

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
            "X-Plex-Platform",
            header::HeaderValue::from_str(platform.clone().as_str()).unwrap(),
        );
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/json"),
        );

        Self {
            http_client: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
            host: config.host,
            x_plex_token: token,
            x_plex_client_identifier: client_identifier,
            x_plex_platform: platform,
            cache: CACHE.clone(),
        }
    }
}
