use crate::config::Config;
use crate::models::*;
use crate::proxy::PlexProxy;
use crate::utils::*;
use anyhow::Result;
use http::HeaderValue;
use http::Uri;
use hyper::client::HttpConnector;
use hyper::Body;
use salvo::http::ReqBody;
use salvo::Request;
use salvo::Response;
// use hyper::client::HttpConnector;
use reqwest::Client;
use salvo::http::ResBody;

type HttpClient = hyper::client::Client<HttpConnector, Body>;

type HyperRequest = hyper::Request<ReqBody>;
type HyperResponse = hyper::Response<ResBody>;

#[derive(Debug, Clone)]
pub struct PlexClient {
    pub http_client: HttpClient,
    pub host: String, // TODO: Dont think this suppsoed to be here. Should be higher up
    pub content_type: ContentType,

    // /// `X-Plex-Provides` header value. Comma-separated list.
    // ///
    // /// Should be one or more of `controller`, `server`, `sync-target`, `player`.
    // pub x_plex_provides: String,

    // /// `X-Plex-Platform` header value.
    // ///
    // /// Platform name, e.g. iOS, macOS, etc.
    pub x_plex_platform: String,

    // /// `X-Plex-Platform-Version` header value.
    // ///
    // /// OS version, e.g. 4.3.1
    // pub x_plex_platform_version: String,

    // /// `X-Plex-Product` header value.
    // ///
    // /// Application name, e.g. Laika, Plex Media Server, Media Link.
    // pub x_plex_product: String,

    // /// `X-Plex-Version` header value.
    // ///
    // /// Application version, e.g. 10.6.7.
    // pub x_plex_version: String,

    // /// `X-Plex-Device` header value.
    // ///
    // /// Device name and model number, e.g. iPhone3,2, Motorola XOOM™, LG5200TV.
    // pub x_plex_device: String,

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

    /// `X-Plex-Sync-Version` header value.
    ///
    /// Not sure what are the valid values, but at the time of writing Plex Web sends `2` here.
    pub x_plex_sync_version: String,
}

impl PlexClient {
    // TODO: Handle 404s/500 etc
    pub fn get(&self, path: String) -> hyper::client::ResponseFuture {
        /// Could use this: https://docs.rs/tower-http/latest/tower_http/propagate_header/index.html
        /// https://github.com/tokio-rs/axum/discussions/1131
        let uri = format!("{}{}", self.host, path);
        // dbg!(&uri);
        let request = hyper::Request::builder()
            .uri(uri)
            .header("X-Plex-Client-Identifier", &self.x_plex_client_identifier)
            .header("X-Plex-Token", &self.x_plex_token)
            // .header("Accept", &self.content_type.to_string())
            .header("Accept", "application/json")
            .body(Body::empty())
            .unwrap();
        self.http_client.request(request)
    }

    pub async fn request(&self, req: &mut Request) -> Response {
        let path = req.uri().path();
        let upstream = format!("{}{}", self.host.clone(), path);
        let proxy = PlexProxy::new(upstream);
        proxy.request(req).await
    }

    // pub fn request(&self, req) -> hyper::client::ResponseFuture {
    //     self.http_client.request(req)
    // }

    pub async fn get_section_collections(&self, id: u32) -> Result<Vec<MetaData>> {
        let resp = self
            .get(format!("/library/sections/{}/collections", id))
            .await
            .unwrap();

        let mut container: MediaContainerWrapper<MediaContainer> = from_response_hyper(resp)
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
        let container: MediaContainerWrapper<MediaContainer> = from_response_hyper(resp).await.unwrap();
        Ok(container)
    }

    pub async fn get_collection(&self, id: i32) -> Result<MediaContainerWrapper<MediaContainer>> {
        let resp = self
            .get(format!("/library/collections/{}", id))
            .await
            .unwrap();
        let container: MediaContainerWrapper<MediaContainer> = from_response_hyper(resp).await.unwrap();
        Ok(container)
    }

    pub async fn get_item_by_key(
        self,
        key: String,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let resp = self.get(key).await.unwrap();
        let container: MediaContainerWrapper<MediaContainer> = from_response_hyper(resp).await.unwrap();
        Ok(container)
    }
}

impl PlexClient {
    pub fn new(req: &mut Request, params: PlexParams) -> Self {
        // TODO: Dont need request
        let config: Config = Config::figment().extract().unwrap();
        // dbg!(get_content_type_from_headers(req.headers()));
        Self {
            http_client: HttpClient::new(),
            // host: "http://100.91.35.113:32400".to_string(),
            host: config.host,
            x_plex_token: params.clone()
                .x_plex_token
                .expect("Expected to have an token in header or query"),
            x_plex_client_identifier: params.clone()
                .x_plex_client_identifier
                .expect("Expected to have an plex client identifier header"),
            x_plex_platform: params.clone().platform.unwrap_or_default(),
            x_plex_sync_version: "2".to_owned(),
            content_type: get_content_type_from_headers(req.headers()),
        }
    }
}
