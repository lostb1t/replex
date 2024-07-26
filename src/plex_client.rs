use std::time::Duration;

use crate::config::Config;
use crate::models::*;
use crate::utils::*;
use anyhow::Result;
use std::collections::HashMap;

use crate::cache::GLOBAL_CACHE;
use async_recursion::async_recursion;
use futures_util::Future;
use futures_util::TryStreamExt;
use http::header::ACCEPT_LANGUAGE;
use http::header::CONNECTION;
use http::header::COOKIE;
use http::header::FORWARDED;
use http::HeaderMap;
use http::Uri;
// use hyper::client::HttpConnector;
// use hyper::Body;
use moka::future::Cache;
//use moka::future::ConcurrentCacheExt;
use once_cell::sync::Lazy;
use reqwest::header;
use reqwest::header::ACCEPT;
use reqwest_retry::{
    default_on_request_failure, Retryable, RetryableStrategy,
};
use salvo::Error;
use salvo::Request;
// use hyper::client::HttpConnector;


static CACHE: Lazy<Cache<String, MediaContainerWrapper<MediaContainer>>> =
    Lazy::new(|| {
        let c: Config = Config::figment().extract().unwrap();
        Cache::builder()
            .max_capacity(10000)
            .time_to_live(Duration::from_secs(c.cache_ttl))
            .eviction_listener(|key, value, cause| {
                //println!("Evicted ({key:?},{value:?}) because {cause:?}")
            })
            .build()
    });

struct Retry401;
impl RetryableStrategy for Retry401 {
    fn handle(
        &self,
        res: &std::result::Result<reqwest::Response, reqwest_middleware::Error>,
    ) -> Option<Retryable> {
        match res {
            Ok(success) if success.status() == 401 => {
                Some(Retryable::Transient)
            }
            Ok(success) => None,
            // otherwise do not retry a successful request
            Err(error) => default_on_request_failure(error),
        }
    }
}

/// TODO: Implement clone
#[derive(Debug, Clone)]
pub struct PlexClient {
    pub http_client: reqwest_middleware::ClientWithMiddleware,
    pub context: PlexContext,
    pub host: String, // TODO: Dont think this supposed to be here. Should be higher up
    pub cache: Cache<String, MediaContainerWrapper<MediaContainer>>,
    pub default_headers: header::HeaderMap,
}

impl PlexClient {
    // TODO: Handle 404s/500 etc
    // TODO: Map reqwest response and error to salvo
    pub async fn get(&self, path: String) -> Result<reqwest::Response, Error> {
        let mut req = Request::default();
        *req.method_mut() = http::Method::GET;
        req.set_uri(Uri::builder().path_and_query(path).build().unwrap());
        self.request(&mut req).await
    }

    pub async fn request(
        &self,
        req: &Request,
    ) -> Result<reqwest::Response, Error> {
        let url = format!(
            "{}{}",
            self.host,
            &req.uri().clone().path_and_query().unwrap()
        );
        let mut headers = self.default_headers.clone();
        for (key, value) in req.headers().iter() {
            if key != ACCEPT && key != http::header::HOST {
              headers.insert(key, value.clone());
            }
        }
        //let mut headers = req.headers_mut().clone();
        //headers.remove(ACCEPT); // remove accept as we always do json request
        //dbg!(&headers);
        //dbg!(&url);
        let res = self
            .http_client
            .request(req.method().clone(), url)
            .headers(headers)
            .send()
            .await
            .map_err(Error::other)?;

        Ok(res)
    }

    pub async fn proxy_request(
         &self,
         req: &Request,
     ) -> Result<reqwest::Response, Error> {
        let url = format!(
            "{}{}?{}",
            self.host,
            encode_url_path(&url_path_getter(req).unwrap()),
            url_query_getter(req).unwrap()
        );
        //dbg!(&req);
        //dbg!(&url);
        //dbg!(&req.uri().clone().query().unwrap().to_string());
        let mut headers = req.headers().clone();
        headers.remove(ACCEPT); // remove accept as we always do json request
        headers.remove(http::header::HOST);
        let res = self
            .http_client
            .request(req.method().clone(), url)
            //.execute(req)
            .headers(headers)
            .send()
            .await
            .map_err(Error::other)?;
        //dbg!(&res);
        Ok(res)
     }

    pub async fn get_section_collections(
        &self,
        id: i64,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let res = self
            .get(format!("/library/sections/{}/collections", id))
            .await
            .unwrap();

        let container: MediaContainerWrapper<MediaContainer> =
            from_reqwest_response(res)
                .await
                .expect("Cannot get MediaContainerWrapper from response");

        Ok(container)
    }

    pub async fn get_collection_children(
        &self,
        id: i64,
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

        // we want guids for banners
        path = format!("{}&includeGuids=1", path);
        // dbg!(&path);

        let res = self.get(path).await.unwrap();
        if !res.status().is_success() {
            return Err(anyhow::anyhow!(format!(
                "unexpected status code: status = {}",
                res.status()
            )));
        }
        
        let container: MediaContainerWrapper<MediaContainer> =
            from_reqwest_response(res).await.unwrap();
        Ok(container)
    }

    #[async_recursion]
    pub async fn load_collection_children_recursive(
        &self,
        id: i64,
        offset: i32,
        limit: i32,
        original_limit: i32,
    ) -> anyhow::Result<MediaContainerWrapper<MediaContainer>> {
        let config: Config = Config::figment().extract().unwrap();
        let mut c = self
            .get_collection_children(id, Some(offset), Some(limit))
            .await?;
        c.media_container.children_mut().retain(|x| !x.is_watched());
        c.media_container
            .children_mut()
            .truncate(original_limit as usize);

        Ok(c)
    }

    pub async fn get_collection(
        &self,
        id: i32,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let res = self.get(format!("/library/collections/{}", id)).await?;

        if res.status() == 404 {
            return Err(salvo::http::StatusError::not_found().into());
        }

        let container: MediaContainerWrapper<MediaContainer> =
            from_reqwest_response(res).await.unwrap();
        Ok(container)
    }

    // theres actually a global endpoint https://plex.sjoerdarendsen.dev/library/all?show.collection=2042780&collection=2042780&X-Plex-Container-Start=0&X-Plex-Container-Size=72
    pub async fn get_collection_total_size_unwatched(
        &self,
        section_id: i32,
        collection_index: i32,
        r#type: String,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let mut path = format!("/library/sections/{}/all?X-Plex-Container-Start=0&X-Plex-Container-Size=0", section_id);
        // dbg!(&path);

        if r#type == "show" {
            path = format!(
                "{}&show.unwatchedLeaves=1&show.collection={}",
                path, collection_index
            );
        }

        if r#type == "movie" {
            path = format!(
                "{}&movie.unwatched=1&movie.collection={}",
                path, collection_index
            );
        }
        // dbg!(&path);
        let res = self.get(path).await?;

        if res.status() == 404 {
            return Err(salvo::http::StatusError::not_found().into());
        }

        let container: MediaContainerWrapper<MediaContainer> =
            from_reqwest_response(res).await.unwrap();
        Ok(container)
    }

    pub async fn get_hubs(
        &self,
        id: i32,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let res = self.get("/hubs".to_string()).await.unwrap();
        let container: MediaContainerWrapper<MediaContainer> =
            from_reqwest_response(res).await.unwrap();
        Ok(container)
    }

    pub async fn get_item_by_key(
        self,
        key: String,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let res = self.get(key).await.unwrap();
        let container: MediaContainerWrapper<MediaContainer> =
            from_reqwest_response(res).await.unwrap();
        Ok(container)
    }

    pub async fn get_cached(
        self,
        f: impl Future<Output = Result<MediaContainerWrapper<MediaContainer>>>,
        name: String,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let cache_key = self.generate_cache_key(name.clone());
        let cached = self.get_cache(&cache_key).await?;

        if cached.is_some() {
            return Ok(cached.unwrap());
        }
        let r = f.await?;
        self.insert_cache(cache_key, r.clone()).await;
        Ok(r)
    }

    pub async fn get_hero_art(
        self,
        uuid: String,
    ) -> Option<String> {
        //tracing::debug!(uuid = uuid, "Loading hero art from plex");
        let cache_key = format!("{}:hero_art", uuid);

        let cached_result: Option<Option<String>> =
            GLOBAL_CACHE.get(cache_key.as_str()).await;

        if cached_result.is_some() {
            //tracing::debug!("Returning cached version");
            return cached_result.unwrap();
        }

        let mut container: MediaContainerWrapper<MediaContainer> =
            match self.get_provider_data(&uuid).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::warn!(
                        uuid = uuid,
                        error = %e,
                        "Problem loading provider metadata."
                    );
                    return None;
                    MediaContainerWrapper::default()
                }
            };
    
        let metadata = container.media_container.children_mut().get(0);
        let mut image: Option<String> = None;
        if metadata.is_some() {
            for i in &metadata.unwrap().images {
                if i.r#type == "coverArt" {
                    image = Some(i.url.clone());
                    break;
                }
            }
        }
        
        if image.is_none() {
           tracing::warn!(uuid = uuid, "No hero image found on plex");
        }
        
        image.as_ref()?; // dont return and dont cache, let us just retry next time.

        //tracing::debug!("Hero image found");

        let cache_expiry = crate::cache::Expiration::Month;
        let _ = GLOBAL_CACHE
            .insert(cache_key, image.clone(), cache_expiry)
            .await;

        image
    }

    pub async fn get_provider_data(
        self,
        uuid: &String,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let config: Config = Config::figment().extract().unwrap();
        let url = format!(
            "https://metadata.provider.plex.tv/library/metadata/{}",
            uuid
        );

        let mut req = reqwest::Request::new(
            http::Method::GET,
            url.parse::<url::Url>().unwrap(),
        );
        let mut headers = HeaderMap::new();
        
        //endpoint is buggy, if llex has a cached version then it doesnt need a plex token
        // but if not cached then a server admin token is needed
        let mut token = config.token.clone();
        if token.is_some() {
            headers.insert(
                "X-Plex-Token",
                header::HeaderValue::from_str(token.unwrap().as_str()).unwrap(),
            );
        };

        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/json"),
        );
        *req.headers_mut() = headers;

        let res = self.http_client.execute(req).await.map_err(Error::other)?;

        if res.status() != salvo::http::StatusCode::OK {
            return Err(anyhow::anyhow!(format!(
                "unexpected status code: status = {}",
                res.status()
            )));
        }

        let container: MediaContainerWrapper<MediaContainer> =
            from_reqwest_response(res).await?;
        Ok(container)
    }

    async fn get_cache(
        &self,
        cache_key: &str,
    ) -> Result<Option<MediaContainerWrapper<MediaContainer>>> {
        Ok(self.cache.get(cache_key).await)
    }

    async fn insert_cache(
        &self,
        cache_key: String,
        container: MediaContainerWrapper<MediaContainer>,
    ) {
        self.cache.insert(cache_key, container).await;
    }

    fn generate_cache_key(&self, name: String) -> String {
        format!("{}:{}", name, self.context.token.clone().unwrap())
    }

    pub fn from_context(context: &PlexContext) -> Self {
        let config: Config = Config::figment().extract().unwrap();
        let token = context
            .clone()
            .token
            .expect("Expected to have an token in header or query");
        let client_identifier = context.clone().client_identifier;
        let platform = context.platform.clone().unwrap_or_default();

        //let req_headers = req.headers().clone();
        let mut headers = header::HeaderMap::new();
        let headers_map = HashMap::from([
            ("X-Plex-Token", context.token.clone()),
            ("X-Plex-Platform", Some(platform.clone().to_string())),
            ("X-Plex-Client-Identifier", context.client_identifier.clone()),
            ("X-Plex-Session-Id", context.session_id.clone()),
            ("X-Plex-Playback-Session-Id", context.playback_session_id.clone()),
            ("X-Plex-Product", context.product.clone()),
            ("X-Plex-Playback-Id", context.playback_id.clone()),
            ("X-Plex-Platform-Version", context.platform_version.clone()),
            ("X-Plex-Version", context.version.clone()),
            ("X-Plex-Features", context.features.clone()),
            ("X-Plex-Model", context.model.clone()),
            ("X-Plex-Device", context.device.clone()),
            ("X-Plex-Device-Name", context.device_name.clone()),
            ("X-Plex-Drm", context.drm.clone()),
            ("X-Plex-Text-Format", context.text_format.clone()),
            ("X-Plex-Http-Pipeline", context.http_pipeline.clone()),
            ("X-Plex-Provider-Version", context.provider_version.clone()),
            ("X-Plex-Device-Screen-Resolution", context.screen_resolution_original.clone()),
            ("X-Plex-Client-Capabilities", context.client_capabilities.clone()),
            ("X-Forwarded-For", context.forwarded_for.clone()),
            ("X-Real-Ip", context.real_ip.clone()),
            (&ACCEPT.as_str(), Some("application/json".to_string())),
            (&ACCEPT_LANGUAGE.as_str(), Some("en-US".to_string())),
            //(http::header::HOST.as_str(), Some(config.host.clone().unwrap())),
        ]);
        
        for (key, val) in headers_map {
            if val.is_some() {
              headers.insert(key.clone(), val.unwrap().as_str().parse().unwrap());
            }
        }
        
       //let target_uri: url::Url = url::Url::parse(config.host.clone().unwrap().as_str()).unwrap();
       //let target_host = target_uri.host().unwrap().to_string().clone();

        //headers.insert(
        //    http::header::HOST,
        //    header::HeaderValue::from_str(&target_host).unwrap(),
        //);

        Self {
            http_client: reqwest_middleware::ClientBuilder::new(
                reqwest::Client::builder()
                    //.default_headers(headers)
                    .gzip(true)
                    .timeout(Duration::from_secs(30))
                    .build()
                    .unwrap(),
            )
            .build(),
            default_headers: headers,
            host: config.host.unwrap(),
            context: context.clone(),
            //x_plex_token: token,
            //x_plex_client_identifier: client_identifier,
            //x_plex_platform: platform,
            cache: CACHE.clone(),
        }
    }

    // pub fn dummy() -> Self {
    //     let config: Config = Config::figment().extract().unwrap();
    //     let token = "DUMMY".to_string();
    //     let client_identifier: Option<String> = None;
    //     let platform: Platform = Platform::Generic;

    //     // Dont do the headers here. Do it in prepare function
    //     let mut headers = header::HeaderMap::new();
    //     headers.insert(
    //         "X-Plex-Token",
    //         header::HeaderValue::from_str(token.clone().as_str()).unwrap(),
    //     );
    //     headers.insert(
    //         "Accept",
    //         header::HeaderValue::from_static("application/json"),
    //     );
    //     headers.insert(
    //         "X-Plex-Platform",
    //         header::HeaderValue::from_str(platform.to_string().as_str())
    //             .unwrap(),
    //     );
    //     Self {
    //         http_client: reqwest::Client::builder()
    //             .default_headers(headers)
    //             .gzip(true)
    //             .timeout(Duration::from_secs(30))
    //             .build()
    //             .unwrap(),
    //         host: config.host.unwrap(),
    //         x_plex_token: token,
    //         x_plex_client_identifier: client_identifier,
    //         x_plex_platform: platform,
    //         cache: CACHE.clone(),
    //     }
    // }
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
