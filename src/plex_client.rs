use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::models::*;
use crate::proxy::Proxy;
use crate::utils::*;
use anyhow::Result;

use async_recursion::async_recursion;
use futures_util::Future;
use futures_util::TryStreamExt;
use http::Uri;
use salvo::proxy::Upstreams;
use salvo::BoxedError;
// use hyper::client::HttpConnector;
// use hyper::Body;
use hyper::body::Body;
use moka::future::Cache;
use moka::future::ConcurrentCacheExt;
use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use reqwest::header;
use reqwest::header::HeaderValue;
use reqwest::header::ACCEPT;
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

/// TODO: Implement clone
#[derive(Debug, Clone)]
pub struct PlexClient {
    pub http_client: Client,
    // pub host: String, // TODO: Dont think this suppsoed to be here. Should be higher up
    pub cache: Cache<String, MediaContainerWrapper<MediaContainer>>,

    // /// `X-Plex-Platform` header value.
    // ///
    // /// Platform name, e.g. iOS, macOS, etc.
    pub x_plex_platform: Platform,

    /// `X-Plex-Client-Identifier` header value.
    ///
    /// UUID, serial number, or other number unique per device.
    ///
    /// **N.B.** Should be unique for each of your devices.
    pub x_plex_client_identifier: Option<String>,

    /// `X-Plex-Token` header value.
    ///
    /// Auth token for Plex.
    pub x_plex_token: String,

    pub proxy: Proxy,

    // other non eseential headers
    pub headers: Option<header::HeaderMap>,
}

// impl PlexClient {
impl PlexClient {
    // TODO: Handle 404s/500 etc
    // TODO: Map reqwest response and error to salvo
    pub async fn get(&self, path: String) -> Result<Response, Error> {
        // let uri = format!("{}{}", self.host, path);
        let mut req = Request::default();
        *req.method_mut() = http::Method::GET;
        if self.headers.is_some() {
            *req.headers_mut() = self.headers.clone().unwrap()
        }
        // dbg!(&self.headers);
        req.set_uri(Uri::builder().path_and_query(path).build().unwrap());
        self.request(&mut req).await
    }

    // pub async fn request(
    //     &self,
    //     req: &mut Request,
    // ) -> Result<reqwest::Response, Error> {
    //     let config: Config = Config::figment().extract().unwrap();
    //     let proxy = Proxy::with_client(
    //         config.host.clone().unwrap(),
    //         reqwest::Client::builder()
    //             .timeout(Duration::from_secs(30))
    //             .build()
    //             .unwrap(),
    //     );

    //     let res = proxy.request(req).await.unwrap();
    //     let reqwest_res = reqwest::Response::
    //     Ok(reqwest_res)
    // }

    pub async fn request(&self, req: &mut Request) -> Result<Response, Error> {
        // let uri = format!(
        //     "{}{}",
        //     self.host,
        //     &req.uri_mut().path_and_query().unwrap()
        // );
        // dbg!(&req);

        req.headers_mut().remove(ACCEPT); // remove accept as we always do json request
        req.headers_mut().insert(
            ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        let res = self.proxy.request(req).await?;
        Ok(res)
    }

    // pub async fn request(
    //     &self,
    //     req: &mut Request,
    // ) -> Result<reqwest::Response, Error> {
    //     let uri = format!(
    //         "{}{}",
    //         self.host,
    //         &req.uri_mut().path_and_query().unwrap()
    //     );
    //     let mut headers = req.headers_mut().to_owned();
    //     let target_uri: url::Url = url::Url::parse(self.host.as_str()).unwrap();
    //     let target_host = target_uri.host().unwrap().to_string().clone();

    //     headers.remove(ACCEPT); // remove accept as we always do json request
    //     headers.insert(
    //         http::header::HOST,
    //         header::HeaderValue::from_str(
    //             &target_host,
    //         ).unwrap(),
    //     );

    //     let mut url = url::Url::parse(req.uri_mut().to_string().as_str()).unwrap();
    //     url.set_host(Some(self.host.replace("http://", "").replace("https://", "").as_str())).unwrap();
    //     url.set_scheme(target_uri.scheme()).unwrap();
    //     url.set_port(target_uri.port()).unwrap();
    //     req.set_uri(hyper::Uri::try_from(url.as_str()).unwrap());

    //     let res = self
    //         .http_client
    //         .request(req.method_mut().to_owned(), uri)
    //         .headers(headers)
    //         .send()
    //         .await
    //         .map_err(Error::other)?;
    //     // let res = self
    //     //     .http_client
    //     //     .get(uri)
    //     //     .headers(headers)
    //     //     .send()
    //     //     .await
    //     //     .map_err(Error::other)?;
    //     Ok(res)
    // }

    pub async fn get_section_collections(
        &self,
        id: i64,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let res = self
            .get(format!("/library/sections/{}/collections", id))
            .await
            .unwrap();

        let mut container: MediaContainerWrapper<MediaContainer> =
            from_salvo_response(res)
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

        let resp = self.get(path).await.unwrap();
        let container: MediaContainerWrapper<MediaContainer> =
            from_salvo_response(resp).await.unwrap();
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
        // if !config.include_watched {
        //     let original_size = c.media_container.size.unwrap();
        //     //let children =
        //     c.media_container.children_mut().retain(|x| !x.is_watched());
        //     let children_lenght = c.media_container.children_mut().len() as i32;
        //     let total_size = c.media_container.total_size.unwrap();

        //     // dbg!("checking", original_size, children_lenght, offset, total_size, limit, original_limit);
        //     // if original_size != children_lenght {
        //     //     dbg!("checking", original_size, children_lenght, offset, total_size, limit, original_limit);
        //     // }
        //     // take into account watched, reload if we are under the requested limit
        //     if (children_lenght < original_limit
        //         && total_size > offset + limit
        //         && offset < total_size)
        //         || (children_lenght < original_limit
        //             && total_size > offset + original_size)
        //     {
        //         let mut children = c.media_container.children();
        //         // dbg!("recursive");
        //         // dbg!("checking", original_size, children_lenght, offset, total_size, limit);
        //         let new_limit = match limit {
        //             x if x < 25 => 25 + x,
        //             x if x > 25 => x * 2,
        //             _ => 25 + limit,
        //         };
        //         let mut r = self
        //             .load_collection_children_recursive(
        //                 id,
        //                 offset + children_lenght,
        //                 new_limit,
        //                 // limit + 10,
        //                 original_limit,
        //             )
        //             .await.unwrap();
        //         children.append(r.media_container.children_mut());
        //         c.media_container.set_children(children);
        //         // return self
        //         //     .load_collection_children_recursive(
        //         //         id,
        //         //         offset,
        //         //         new_limit,
        //         //         // limit + 10,
        //         //         original_limit,
        //         //     )
        //         //     .await;
        //     }
        // }
        // dbg!(c.media_container.children_mut().len());
        // dbg!(limit);
        c.media_container
            .children_mut()
            .truncate(original_limit as usize);

        // for (pos, child) in c.media_container.children().iter().enumerate() {
        //     if child.title == "Plane" {
        //         dbg!(pos);
        //     }
        // }
        // dbg!(c.media_container.children_mut().len());
        // dbg!("-----");
        // dbg!("checking", offset, c.media_container.total_size.unwrap(), limit, original_limit, c.media_container.children_mut().len());
        Ok(c)
    }

    pub async fn get_collection(
        &self,
        id: i32,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let res = self.get(format!("/library/collections/{}", id)).await?;

        if res.status_code.unwrap() == http::StatusCode::NOT_FOUND {
            return Err(salvo::http::StatusError::not_found().into());
        }

        let container: MediaContainerWrapper<MediaContainer> =
            from_salvo_response(res).await.unwrap();
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

        if res.status_code.unwrap() == http::StatusCode::NOT_FOUND {
            return Err(salvo::http::StatusError::not_found().into());
        }

        let container: MediaContainerWrapper<MediaContainer> =
            from_salvo_response(res).await.unwrap();
        Ok(container)
    }

    pub async fn get_hubs(
        &self,
        id: i32,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let resp = self.get("/hubs".to_string()).await.unwrap();
        let container: MediaContainerWrapper<MediaContainer> =
            from_salvo_response(resp).await.unwrap();
        Ok(container)
    }

    pub async fn get_item_by_key(
        self,
        key: String,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let resp = self.get(key).await.unwrap();
        let container: MediaContainerWrapper<MediaContainer> =
            from_salvo_response(resp).await.unwrap();
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

    pub async fn get_provider_data(
        self,
        guid: String,
    ) -> Result<MediaContainerWrapper<MediaContainer>> {
        let uri = format!(
            "https://metadata.provider.plex.tv/library/metadata/{}",
            guid
        );

        // we dont want the default headers and client as we aint talking to a plex server
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "X-Plex-Token",
            header::HeaderValue::from_str(self.x_plex_token.clone().as_str())
                .unwrap(),
        );
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/json"),
        );

        let client = reqwest::Client::builder()
                // .default_headers(headers.clone())
                .gzip(true)
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap();
    
        let res = client
            .get(uri)
            .headers(headers)
            .send()
            .await
            .map_err(Error::other)?;
        // dbg!(&res.status());
        // dbg!(res.status());
        // if res.status() == 404 {
        //     return Err(salvo::http::StatusError::not_found().into());
        // }

        // if res.status() == 500 {
        //     return Err(salvo::http::StatusError::);
        // }

        let container: MediaContainerWrapper<MediaContainer> =
            from_reqwest_response(res).await?;
        Ok(container)
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

    pub fn from_request(req: &Request, params: PlexContext) -> Self {
        let config: Config = Config::figment().extract().unwrap();
        let token = params
            .clone()
            .token
            .expect("Expected to have an token in header or query");
        let client_identifier = params.clone().client_identifier;
        let platform = params.clone().platform;

        //let mut headers = header::HeaderMap::new();
        let mut headers = req.headers().clone();

        // lets copy x-plex querys to the header. Otherwise we have to handle queries and headers.
        let request_url =
            url::Url::parse(req.uri().to_string().as_str()).unwrap();
        for query in request_url.query_pairs() {
            let key = query.0.clone().into_owned().to_lowercase();
            let val = query.1.clone().into_owned();
            if key.starts_with("x-plex") {
                // dbg!(key.clone());
                headers.insert(
                    header::HeaderName::from_static(Box::leak(
                        key.into_boxed_str(),
                    )),
                    header::HeaderValue::from_str(val.clone().as_str())
                        .unwrap(),
                );
            }
        }

        headers.insert(
            "X-Plex-Token",
            header::HeaderValue::from_str(token.clone().as_str()).unwrap(),
        );
        if let Some(i) = client_identifier.clone() {
            headers.insert(
                "X-Plex-Client-Identifier",
                header::HeaderValue::from_str(i.as_str()).unwrap(),
            );
        }
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "X-Plex-Platform",
            header::HeaderValue::from_str(platform.to_string().as_str())
                .unwrap(),
        );

        let config: Config = Config::figment().extract().unwrap();
        let proxy = Proxy::with_client(
            config.host.clone().unwrap(),
            reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
        );
        // dbg!(&headers);
        Self {
            http_client: reqwest::Client::builder()
                .default_headers(headers.clone())
                .gzip(true)
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            // host: config.host.unwrap(),
            x_plex_token: token,
            x_plex_client_identifier: client_identifier,
            x_plex_platform: platform,
            headers: Some(headers.clone()),
            proxy: proxy,
            cache: CACHE.clone(),
        }
    }

    pub fn dummy() -> Self {
        let config: Config = Config::figment().extract().unwrap();
        let token = "DUMMY".to_string();
        let client_identifier: Option<String> = None;
        let platform: Platform = Platform::Generic;

        // Dont do the headers here. Do it in prepare function
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "X-Plex-Token",
            header::HeaderValue::from_str(token.clone().as_str()).unwrap(),
        );
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "X-Plex-Platform",
            header::HeaderValue::from_str(platform.to_string().as_str())
                .unwrap(),
        );
        let config: Config = Config::figment().extract().unwrap();
        let proxy = Proxy::with_client(
            config.host.clone().unwrap(),
            reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
        );
        Self {
            http_client: reqwest::Client::builder()
                .default_headers(headers)
                .gzip(true)
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            // host: config.host.unwrap(),
            x_plex_token: token,
            x_plex_client_identifier: client_identifier,
            x_plex_platform: platform,
            proxy: proxy,
            headers: None,
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
