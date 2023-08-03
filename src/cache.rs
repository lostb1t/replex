use async_trait::async_trait;
use salvo::{cache::CacheIssuer, Request, Depot};

pub struct RequestIssuer {
    use_scheme: bool,
    use_authority: bool,
    use_path: bool,
    use_query: bool,
    use_method: bool,
    use_token: bool
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
    async fn issue(&self, req: &mut Request, _depot: &Depot) -> Option<Self::Key> {
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