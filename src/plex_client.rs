use crate::config::Config;
use crate::models::*;
use crate::proxy::PlexProxy;
use crate::utils::*;
use anyhow::Result;

use futures_util::Future;
use futures_util::TryStreamExt;
// use hyper::client::HttpConnector;
// use hyper::Body;
use http_cache_reqwest::{CACacheManager, MokaManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use hyper::body::Body;
use memoize::memoize;
use reqwest::header;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use salvo::http::ReqBody;
use salvo::Error;
use salvo::Request;
use salvo::Response;
// use hyper::client::HttpConnector;

use salvo::http::ResBody;
use tracing::debug;

// type HttpClient = hyper::client::Client<HttpConnector, Body>;

#[derive(Debug, Clone)]
pub struct PlexClient {
    pub http_client: ClientWithMiddleware,
    pub host: String, // TODO: Dont think this suppsoed to be here. Should be higher up
    // pub http_moka_cache: HTTPMokaCache,

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
    pub fn format_url(&self, path: String) -> String {
        format!("{}{}", self.host, path)
    }

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

    // #[memoize]
    // pub async fn get_section_labels(&self, section_id: u32) -> Result<Vec<Label>> {
    //     debug!("getting labels");
    //     // type Test = MediaContainerWrapper<Label>;
    //     let res: MediaContainerWrapper<Label> = self
    //         .http_client
    //         .get(format!("/library/sections/{}/label", section_id))
    //         .send()
    //         .await?
    //         .json::<MediaContainerWrapper<Label>>()
    //         .await?;
    //     // Ok(res.media_container)
    //     // let mut container: MediaContainerWrapper<MediaContainer> = from_reqwest_response(res)
    //     //     .await
    //     //     .expect("Cannot get MediaContainerWrapper from response");

    //     // Ok(container.media_container.children())
    // }
    // pub fn request(&self, req) -> hyper::client::ResponseFuture {
    //     self.http_client.request(req)
    // }

    pub async fn get_section_collections(&self, id: u32) -> Result<Vec<MetaData>> {
        debug!("getting collections");
        let res = self
            .get(format!("/library/sections/{}/collections", id))
            .await
            .unwrap();

        let mut container: MediaContainerWrapper<MediaContainer> = from_reqwest_response(res)
            .await
            .expect("Cannot get MediaContainerWrapper from response");

        Ok(container.media_container.children())
    }

    pub async fn get_collection_children(
        &self,
        id: u32,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let mut path = format!("/library/collections/{}/children", id);

        if offset.is_some() {
            path = format!("{}?X-Plex-Container-Start={}", path, offset.unwrap());
        }
        if limit.is_some() {
            path = format!("{}&X-Plex-Container-Size={}", path, limit.unwrap());
        }
        let resp = self.get(path).await.unwrap();
        let container: MediaContainerWrapper<MediaContainer> =
            from_reqwest_response(resp).await.unwrap();
        Ok(container)
    }

    pub async fn get_collection(&self, id: i32) -> Result<MediaContainerWrapper<MediaContainer>> {
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
}

impl PlexClient {
    pub fn new(req: &mut Request, params: PlexParams, http_moka_cache: HTTPMokaCache) -> Self {
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
            header::HeaderValue::from_str(client_identifier.clone().as_str()).unwrap(),
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
            http_client: ClientBuilder::new(
                reqwest::Client::builder()
                    .default_headers(headers)
                    .build()
                    .unwrap(),
            )
            .with(Cache(HttpCache {
                mode: CacheMode::Default,
                manager: MokaManager::new(http_moka_cache),
                options: HttpCacheOptions::default(),
            }))
            .build(),
            host: config.host,
            x_plex_token: token,
            x_plex_client_identifier: client_identifier,
            x_plex_platform: platform,
            // content_type: get_content_type_from_headers(req.headers()),
        }
    }
}
