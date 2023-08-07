use figment::{providers::Env, Figment};
use serde::Deserialize;

fn default_as_false() -> bool {
    false
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub host: Option<String>,
    #[serde(default = "default_port")]
    pub port: u64,
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub include_watched: bool,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub ssl_enable: bool,
    pub ssl_domain: Option<String>,
    pub newrelic_api_key: Option<String>,
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub enable_console: bool,
    pub tmdb_api_key: Option<String>,
}

fn default_cache_ttl() -> u64 {
    5 * 60
}

fn default_port() -> u64 {
    80
}

impl Config {
    // Note the `nested` option on both `file` providers. This makes each
    // top-level dictionary act as a profile.
    pub fn figment() -> Figment {
        Figment::new().merge(Env::prefixed("REPLEX_"))
    }
    // pub fn default() -> Self {
    //     Config { include_watched: false}
    // }
}
