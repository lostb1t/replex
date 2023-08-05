use std::{sync::Arc, time::SystemTime};

use bytes::Bytes;
use moka::future::Cache;
use reqwest::Url;
use salvo::{async_trait};
use tokio::time::{timeout, Duration};
use crate::config::Config;


/// Data about an entry in the cache
#[derive(Debug, Clone)]
struct CacheEntry {
	/// The body of the cached response
	response: Bytes,
}

impl CacheEntry {
	/// Constructs a new `CacheEntry`.
	pub fn new(response: Bytes) -> Self {
		Self { response }
	}
}

/// Reqwest client with caching abilities
#[derive(Clone, Debug)]
pub struct Client {
    cache: Cache<Url, CacheEntry>,
    inner: reqwest::Client
}

impl Default for Client {
    fn default() -> Self {
        let c: Config = Config::figment().extract().unwrap();
        Client {
            cache: Cache::builder()
                .max_capacity(10000)
                .time_to_live(Duration::from_secs(c.cache_ttl))
                .build(),
            inner: reqwest::Client::new(),
        }
    }
}

// #[async_trait]
impl Client {
	/// Constructs a new `CacheMiddleware`
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_reqwest_client(client: reqwest::Client) -> Self {
		Client { inner: client, ..Client::default() }
	}

    pub fn get<U: reqwest::IntoUrl>(&self, url: U, cache_ttl: Option<Duration>) -> reqwest::RequestBuilder {
		let cache_ttl = cache_ttl.unwrap_or(Duration::from_secs(0));
        self.request(reqwest::Method::GET, url, Some(cache_ttl))
    }

    pub fn request<U: reqwest::IntoUrl>(&self, method: reqwest::Method, url: U, cache_ttl: Option<Duration>) -> reqwest::RequestBuilder {
        self.inner.request(method, url)
    }

	// async fn handle(
	// 	&self,
	// 	mut req: reqwest::Request,
	// 	extensions: &mut Extensions,
	// 	next: Next<'_>,
	// ) -> reqwest_middleware::Result<reqwest::Response> {
	// 	// Strip the fragment part (the stuff after #) of the URL since is exclusively
	// 	// client-side and has no bearing on caching
	// 	let mut url = req.url().clone();
	// 	url.set_fragment(None);
    //     let response = next.run(req, extensions).await?;
	// 	// if let Some(mut cache) = self.cache.get(&url).await {

	// 	// }
	// 	// Make a `Parts` so that we have something to give the `CachePolicy`
	// 	// constructor
	// 	// #[allow(clippy::expect_used)]
	// 	// let (mut parts, _) = http::Request::builder()
	// 	// 	.uri(req.uri())
	// 	// 	.method(req.method().clone())
	// 	// 	.version(req.version())
	// 	// 	.body(())
	// 	// 	.expect("Builder used correctly")
	// 	// 	.into_parts();

	// 	Ok(response)
	// }

	// #[must_use]
	// pub fn with_options(options: CacheOptions) -> Self {
	// 	Self { cache: Arc::new(CHashMap::new()), options }
	// }
}

// impl std::fmt::Debug for CacheMiddleware {
// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// 		f.debug_struct("CacheMiddleware")
// 			.field("cache", &format!("<{} entries>", self.cache.len()))
// 			// .field("options", &self.options)
// 			.finish()
// 	}
// }


// async fn reqwest_to_http(
// 	mut response: reqwest::Response,
// ) -> reqwest::Result<http::Response<Bytes>> {
// 	let mut http = http::Response::new(Bytes::new());
// 	*http.status_mut() = response.status();
// 	*http.version_mut() = response.version();
// 	std::mem::swap(http.headers_mut(), response.headers_mut());
// 	*http.body_mut() = response.bytes().await?;
// 	Ok(http)
// }

