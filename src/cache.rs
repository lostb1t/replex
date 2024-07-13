use async_trait::async_trait;
use http::Uri;
use moka::future::Cache as MokaCache;
//use moka::sync::Cache as MokaCacheSync;
//use moka::sync::CacheBuilder as MokaCacheBuilder;
use moka::Expiry;
use crate::headers;
use once_cell::sync::Lazy;
use salvo::cache::CachedBody;
use regex::Regex;
use salvo::cache::MethodSkipper;
use salvo::conn::SocketAddr;
use salvo::handler::Skipper;
use salvo::http::HeaderMap;
use salvo::http::StatusCode;
use salvo::Handler;
use salvo::{cache::CacheIssuer, Depot, Request};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use std::borrow::Borrow;
use std::convert::Infallible;
use std::error::Error as StdError;
use std::hash::Hash;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
// use bincode::{config, Decode, Encode};

use crate::config::Config;

// we close, this is a good example: https://github.com/getsentry/symbolicator/blob/170062d5bc7d4638a3e6af8a564cd881d798f1f0/crates/symbolicator-service/src/caching/memory.rs#L85

pub type CacheKey = String;
pub type CacheValue = (Expiration, Arc<Vec<u8>>);
// pub type CacheValue = Arc<Vec<u8>>;
pub type GlobalCacheType = MokaCache<CacheKey, CacheValue>;

pub(crate) static GLOBAL_CACHE: Lazy<CacheManager> = Lazy::new(|| {
    let expiry = CacheExpiry;

    // let store: GlobalCacheType =
    CacheManager::new(
        MokaCache::builder()
            .max_capacity(100000)
            .expire_after(expiry)
            .build(),
    )
});

/// An enum to represent the expiration of a value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Expiration {
    /// The value never expires.
    Never,
    /// Global TTL from the config
    Global,
    /// Expires after a month
    Month,
    /// Expires after a day
    Day,
}

impl Expiration {
    /// Returns the duration of this expiration.
    pub fn as_duration(&self) -> Option<Duration> {
        let config: Config = Config::figment().extract().unwrap();
        match self {
            Expiration::Never => None,
            Expiration::Global => Some(Duration::from_secs(config.cache_ttl)),
            Expiration::Month => Some(Duration::from_secs(30 * 24 * 60 * 60)),
            Expiration::Day => Some(Duration::from_secs(60 * 60 * 24)),
        }
    }
}

/// An expiry that implements `moka::Expiry` trait. `Expiry` trait provides the
/// default implementations of three callback methods `expire_after_create`,
/// `expire_after_read`, and `expire_after_update`.
///
/// In this example, we only override the `expire_after_create` method.
pub struct CacheExpiry;

impl Expiry<CacheKey, (Expiration, Arc<Vec<u8>>)> for CacheExpiry {
    /// Returns the duration of the expiration of the value that was just
    /// created.
    fn expire_after_create(
        &self,
        _key: &CacheKey,
        value: &(Expiration, Arc<Vec<u8>>),
        _current_time: Instant,
    ) -> Option<Duration> {
        let duration = value.0.as_duration();
        duration
    }
}

#[derive(Clone)]
pub struct CacheManager {
    /// The instance of `moka::future::Cache`
    // pub store: Arc<Cache<String, Arc<Vec<u8>>>>,
    // pub inner: S,
    pub inner: GlobalCacheType,
}

impl CacheManager {
    /// Create a new manager from a pre-configured Cache
    // pub fn new(store: Cache<String, Arc<Vec<u8>>>) -> Self {
    pub fn new(cache: GlobalCacheType) -> Self {
        Self {
            inner: cache, // store: Arc::new(store),
        }
    }
    /// Clears out the entire cache.
    pub async fn clear(&self) -> anyhow::Result<()> {
        self.inner.invalidate_all();
        //self.inner.sync();
        Ok(())
    }

    pub async fn get<T>(&self, cache_key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        match self.inner.get(cache_key).await {
            Some(d) => {
                let result: T = bincode::deserialize(&d.1).unwrap();
                Some(result)
            }
            None => None,
        }
    }

    pub async fn insert<V>(
        &self,
        cache_key: String,
        v: V,
        expires: Expiration,
    ) -> anyhow::Result<()>
    where
        V: Serialize,
    {
        let value = (expires, Arc::new(bincode::serialize(&v)?));
        self.inner.insert(cache_key, value).await;
        //self.inner.sync();
        Ok(())
    }

    pub async fn delete(&self, cache_key: &str) -> anyhow::Result<()> {
        self.inner.invalidate(cache_key).await;
        //self.inner.sync();
        Ok(())
    }
}

pub struct RequestIssuer {
    use_scheme: bool,
    use_authority: bool,
    use_local_addr: bool,
    use_path: bool,
    path_strip_last_segment: bool,
    // TODO: Also make this like headers. So that we can dump unneeded query variables like X-Plex-Client-Identifier and have cross device caching
    use_query: bool,
    use_method: bool,
    use_mime: bool,
    use_headers: Vec<http::HeaderName>,
}
impl Default for RequestIssuer {
    fn default() -> Self {
        Self::new()
    }
}
impl RequestIssuer {
    /// Create a new `RequestIssuer`.
    pub fn new() -> Self {
        Self {
            use_scheme: true,
            use_authority: true,
            use_local_addr: false,
            use_path: true,
            path_strip_last_segment: false,
            use_query: true,
            use_method: true,
            use_mime: false,
            use_headers: vec![],
        }
    }

    pub fn with_plex_defaults() -> Self {
        Self {
            use_scheme: true,
            use_authority: true,
            use_local_addr: false,
            use_path: true,
            path_strip_last_segment: false,
            use_query: true,
            use_method: true,
            use_mime: false,
            use_headers: vec![
                http::header::ACCEPT,
                http::header::ACCEPT_ENCODING,
                headers::PLEX_TOKEN,
                headers::PLEX_LANGUAGE,
                headers::PLEX_CLIENT_IDENTIFIER,
            ],
        }
    }

    /// Whether to use request's uri scheme when generate the key.
    pub fn use_scheme(mut self, value: bool) -> Self {
        self.use_scheme = value;
        self
    }
    /// Whether to use request's uri authority when generate the key.
    pub fn use_authority(mut self, value: bool) -> Self {
        self.use_authority = value;
        self
    }
    pub fn use_local_addr(mut self, value: bool) -> Self {
        self.use_local_addr = value;
        self
    }
    /// Whether to use request's uri path when generate the key.
    pub fn use_path(mut self, value: bool) -> Self {
        self.use_path = value;
        self
    }
    /// Whether to use request's uri query when generate the key.
    pub fn use_query(mut self, value: bool) -> Self {
        self.use_query = value;
        self
    }
    /// Whether to use request method when generate the key.
    pub fn use_method(mut self, value: bool) -> Self {
        self.use_method = value;
        self
    }
    pub fn use_headers(mut self, value: Vec<http::HeaderName>) -> Self {
        self.use_headers = value;
        self
    }
    pub fn path_strip_last_segment(mut self, value: bool) -> Self {
        self.path_strip_last_segment = value;
        self
    }
}

// #[derive(Encode, Decode, PartialEq, Debug)]
// struct Key {

// }

//#[async_trait]
impl CacheIssuer for RequestIssuer {
    type Key = String;
    //async fn issue(&self, req: &mut Request, depot: &Depot) -> Option<Self::Key> {
    //async fn issue(&self, req: &mut Request, depot: &Depot) -> Option<Self::Key> {
    async fn issue(&self, req: &mut Request, _depot: &Depot) -> Option<Self::Key> {
        let mut key = String::new();
        // key.push_str("uri::http://"); // always http as we use local addr
        if self.use_scheme {
            if let Some(scheme) = req.uri().scheme_str() {
                key.push_str(scheme);
                key.push_str("://");
            }
        }
        if self.use_authority {
            if let Some(authority) = req.uri().authority() {
                key.push_str(authority.as_str());
            }
        }
        if self.use_local_addr {
            key.push_str(
                req.local_addr()
                    .to_string()
                    .replace("socket://", "")
                    .as_str(),
            );
        }
        if self.use_path {
            // strip last segment of an path. Used for paths that include tokens
            if self.path_strip_last_segment{
                // let re = Regex::new(format!(r"{}", self.path_regex.clone().unwrap()).as_str()).unwrap();
                // req.uri().path()
                //let k = req.uri().path().split('/').iter().join("/");
                key.push_str(req.uri().path());
            } else {
                key.push_str(req.uri().path());
            }
        }
        // TODO: Clean up query. Not everything needs a cache change.
        if self.use_query {
            if let Some(query) = req.uri().query() {
                key.push('?');
                key.push_str(query);
            }
        }
        if self.use_method {
            key.push_str("|method::");
            key.push_str(req.method().as_str());
        }
        if self.use_mime {
            if let Some(i) = req.first_accept() {
                key.push_str("|mime::");
                key.push_str(i.to_string().as_str());
            }
        }
        if !self.use_headers.is_empty() && !req.headers().is_empty() {
            key.push_str("|headers::");
            for header in self.use_headers.iter() {
                if req.headers().contains_key(header) {
                    key.push_str("::");
                    if header == http::header::ACCEPT {                   
                        if let Some(i) = req.first_accept() {
                            key.push_str(i.to_string().as_str());
                        }
                    } else {
                        key.push_str(header.as_str());
                    }
                    key.push(':');
                    key.push_str(req.header(header).unwrap());
                }
            }
        }
        // dbg!(&key);
        Some(key)
    }
}


#[async_trait]
pub trait CacheStore: Send + Sync + 'static {
    /// Error type for CacheStore.
    type Error: StdError + Sync + Send + 'static;
    /// Key
    type Key: Hash + Eq + Send + Clone + 'static;
    /// Get the cache item from the store.
    async fn load_entry<Q>(&self, key: &Q) -> Option<CachedEntry>
    where
        Self::Key: Borrow<Q>,
        Q: Hash + Eq + Sync;
    /// Save the cache item from the store.
    async fn save_entry(
        &self,
        key: Self::Key,
        data: CachedEntry,
    ) -> Result<(), Self::Error>;
}


#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct CachedEntry {
    /// Response status.
    pub status: Option<StatusCode>,
    /// Response headers.
    pub headers: HeaderMap,
    /// Response body.
    ///
    /// *Notice: If the response's body is streaming, it will be ignored an not cached.
    pub body: CachedBody,

    pub req_headers: HeaderMap,
    pub req_uri: salvo::http::uri::Uri,
    pub req_method: salvo::http::method::Method,
    pub req_local_addr: salvo::conn::addr::SocketAddr
}
impl CachedEntry {
    /// Create a new `CachedEntry`.
    pub fn new(
        status: Option<StatusCode>,
        headers: HeaderMap,
        body: CachedBody,
        req_headers: HeaderMap,
        req_uri: salvo::http::uri::Uri,
        req_method: salvo::http::method::Method,
        req_local_addr: salvo::conn::addr::SocketAddr
    ) -> Self {
        Self {
            status,
            headers,
            body,
            req_headers,
            req_uri,
            req_method,
            req_local_addr
        }
    }

    /// Get the response status.
    pub fn status(&self) -> Option<StatusCode> {
        self.status
    }

    /// Get the response headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get the response body.
    ///
    /// *Notice: If the response's body is streaming, it will be ignored an not cached.
    pub fn body(&self) -> &CachedBody {
        &self.body
    }
}

#[non_exhaustive]
pub struct Cache<S, I> {
    /// Cache store.
    pub store: S,
    /// Cache issuer.
    pub issuer: I,
    /// Skipper.
    pub skipper: Box<dyn Skipper>,
}

impl<S, I> Cache<S, I> {
    /// Create new `Cache`.
    #[inline]
    pub fn new(store: S, issuer: I) -> Self {
        let skipper = MethodSkipper::new().skip_all().skip_get(false);
        Cache {
            store,
            issuer,
            skipper: Box::new(skipper),
        }
    }
    /// Sets skipper and returns new `Cache`.
    #[inline]
    pub fn skipper(mut self, skipper: impl Skipper) -> Self {
        self.skipper = Box::new(skipper);
        self
    }
}


#[async_trait]
impl<S, I> Handler for Cache<S, I>
where
    S: CacheStore<Key = I::Key>,
    I: CacheIssuer,
{
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut salvo::Response,
        ctrl: &mut salvo::FlowCtrl,
    ) {
        if self.skipper.skipped(req, depot) {
            return;
        }
        let key = match self.issuer.issue(req, depot).await {
            Some(key) => key,
            None => {
                return;
            }
        };

        /// request can be manipulated in handlers. So set this before handlers.
        let req_headers = req.headers().clone();
        let req_uri = req.uri().clone();
        let req_method = req.method().clone();
        let req_local_addr = req.local_addr().clone();

        let cache = match self.store.load_entry(&key).await {
            Some(cache) => { 
                tracing::debug!("returning response from cache");
                cache
            },
            None => {
                ctrl.call_next(req, depot, res).await;
                if !res.body.is_stream() && !res.body.is_error() {
                    let headers = res.headers().clone();
                    let body = TryInto::<CachedBody>::try_into(&res.body);
                    match body {
                        Ok(body) => {
                            let cached_data = CachedEntry::new(
                                res.status_code,
                                headers,
                                body,
                                req_headers,
                                req_uri,
                                req_method,
                                req_local_addr,
                            );
                            if let Err(e) =
                                self.store.save_entry(key, cached_data).await
                            {
                                tracing::error!(error = ?e, "cache failed");
                            }
                        }
                        Err(e) => tracing::error!(error = ?e, "cache failed"),
                    }
                }
                return;
            }
        };
        let CachedEntry {
            status,
            headers,
            body,
            req_headers,
            req_uri,
            req_method,
            req_local_addr,
        } = cache;
        if let Some(status) = status {
            res.status_code(status);
        }
        *res.headers_mut() = headers;
        *res.body_mut() = body.into();
        ctrl.skip_rest();
    }
}
