use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::models::*;
use crate::utils::*;
use crate::plex_client::*;
use anyhow::Result;
use michie::memoized;
use std::collections::HashMap;
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

pub struct PlexApi {
    pub client: PlexClient,
    // pub sections: Option<Vec<MediaContainerWrapper<MediaContainer>>>,
    // pub cache: Cache<String, MediaContainerWrapper<MediaContainer>>,
}

impl PlexApi {

    #[memoized(key_expr = "sections".to_string(), store_type = HashMap<String, MediaContainerWrapper<MediaContainer>>)]
    pub async fn sections(&self) -> MediaContainerWrapper<MediaContainer> {
        dbg!("yooo");
        let mut res = self
            .client
            .get("/library/sections".to_string())
            .await
            .unwrap();
        let container: MediaContainerWrapper<MediaContainer> =
            from_reqwest_response(res).await.unwrap();
        dbg!("tes");
        container
    }

    pub async fn plexherokey(
        &self,
        section_id: i32,
        plex_client: PlexClient,
    ) -> Option<String> {
        let mut labels = self
            .client
            .clone()
            .get_cached_anonymous(
                plex_client.get_section_labels(section_id),
                format!("collection:{}", section_id.clone()).to_string(),
            )
            .await
            .unwrap();
        for label in labels.media_container.children() {
            if label.title == "REPLEXHERO" {
                return Some(label.key);
            }
        }
        None
    }
    // async fn get_cache(
    //     &self,
    //     cache_key: &str,
    // ) -> Result<Option<MediaContainerWrapper<MediaContainer>>> {
    //     Ok(self.cache.get(cache_key))
    // }

    // async fn insert_cache(
    //     &self,
    //     cache_key: String,
    //     container: MediaContainerWrapper<MediaContainer>,
    // ) {
    //     self.cache.insert(cache_key, container).await;
    //     self.cache.sync();
    // }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::*;
    use salvo::prelude::*;
    use salvo::test::{ResponseExt, TestClient};
    use rstest::rstest;
    use std::env;
    use super::*;

    #[tokio::test]
    async fn test_api_sections() {
        // This should be a global. No need to redifine
        let mock_server = get_mock_server();
        env::set_var(
            "REPLEX_HOST",
            format!("http://{}", mock_server.address().to_string()),
        );

        let api = PlexApi {
            client: PlexClient::dummy()
        };
        let result = api.sections().await;
        dbg!("wiiittt");
        dbg!(result);
        assert_eq!("sup", "Hello world!");
    }
}
