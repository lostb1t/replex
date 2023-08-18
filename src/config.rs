use crate::models::deserialize_comma_seperated_string;
use figment::{providers::Env, Figment};
use serde::Deserialize;

fn default_as_false() -> bool {
    false
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub host: Option<String>,
    pub port: Option<u64>,
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub include_watched: bool,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
    #[serde(
        default = "default_as_true",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub cache_refresh: bool,
    #[serde(
        default,
        deserialize_with = "deserialize_comma_seperated_string"
    )]
    pub hero_rows: Option<Vec<String>>,
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
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub disable_user_state: bool,
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub disable_leaf_count: bool,
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub redirect_streams: bool,
    pub redirect_streams_url: Option<String>,
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub disable_related: bool,
}

fn default_cache_ttl() -> u64 {
    15 * 60 // 15 minutes
}

// fn default_port() -> u64 {
//     80
// }

// fn default_hero_rows() -> Option<Vec<String>> {
//     Some(vec![
//         "movies.recent".to_string(),
//         "television.recent".to_string(),
//         "movie.recentlyadded".to_string(),
//     ])
// }

fn default_as_true() -> bool {
    true
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
