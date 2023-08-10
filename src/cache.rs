use async_trait::async_trait;
use moka::{future::Cache, future::ConcurrentCacheExt, Expiry};
use once_cell::sync::Lazy;
use salvo::{cache::CacheIssuer, Depot, Request};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use std::hash::Hash;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use std::error::Error;

use crate::config::Config;

// we close, this is a good example: https://github.com/getsentry/symbolicator/blob/170062d5bc7d4638a3e6af8a564cd881d798f1f0/crates/symbolicator-service/src/caching/memory.rs#L85

pub type CacheKey = String;
pub type CacheValue = (Expiration, Arc<Vec<u8>>);
// pub type CacheValue = Arc<Vec<u8>>;
pub type GlobalCacheType = Cache<CacheKey, CacheValue>;

pub(crate) static GLOBAL_CACHE: Lazy<CacheManager> = Lazy::new(|| {
    let expiry = CacheExpiry;

    // let store: GlobalCacheType =
    CacheManager::new(
        Cache::builder()
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
    /// Expires after a mint
    Month,
}

impl Expiration {
    /// Returns the duration of this expiration.
    pub fn as_duration(&self) -> Option<Duration> {
        let config: Config = Config::figment().extract().unwrap();
        match self {
            Expiration::Never => None,
            Expiration::Global => Some(Duration::from_secs(config.cache_ttl)),
            Expiration::Month => Some(Duration::from_secs(30 * 24 * 60 * 60)),
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
        self.inner.sync();
        Ok(())
    }

    pub async fn get<T>(&self, cache_key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    { 
        match self.inner.get(cache_key) {
            Some(d) => {
                let result: T = bincode::deserialize(&d.1).unwrap();
                Some(result)
            },
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
        // let bytes = bincode::serialize(&value)?;
        self.inner.insert(cache_key, value).await;
        self.inner.sync();
        Ok(())
    }

    pub async fn delete(&self, cache_key: &str) -> anyhow::Result<()> {
        self.inner.invalidate(cache_key).await;
        self.inner.sync();
        Ok(())
    }
}

pub struct RequestIssuer {
    use_scheme: bool,
    use_authority: bool,
    use_path: bool,
    use_query: bool,
    use_method: bool,
    use_token: bool,
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
            use_path: true,
            use_query: true,
            use_method: true,
            use_token: true,
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
    pub fn use_token(mut self, value: bool) -> Self {
        self.use_token = value;
        self
    }
}

#[async_trait]
impl CacheIssuer for RequestIssuer {
    type Key = String;
    async fn issue(
        &self,
        req: &mut Request,
        _depot: &Depot,
    ) -> Option<Self::Key> {
        let mut key = String::new();
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
        if self.use_path {
            key.push_str(req.uri().path());
        }
        if self.use_query {
            if let Some(query) = req.uri().query() {
                key.push('?');
                key.push_str(query);
            }
        }
        if self.use_method {
            key.push('|');
            key.push_str(req.method().as_str());
        }
        if self.use_token {
            // TODO: Implement
            key.push('|');
            key.push_str(req.header("X-Plex-Token").unwrap_or_default());
            if let Some(i) = req.first_accept() {
                key.push('|');
                key.push_str(i.to_string().as_str());
            }
        }
        Some(key)
    }
}
