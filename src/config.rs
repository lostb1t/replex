use crate::models::deserialize_comma_seperated_string;
use figment::{providers::Env, Figment};
use serde::{Deserialize, Deserializer};
// use serde::Deserialize;

fn default_as_false() -> bool {
    false
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_host")]
    pub host: Option<String>,
    pub token: Option<String>,
    pub port: Option<u64>,
    #[serde(
        default = "default_as_true",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub interleave: bool,
    #[serde(
        default = "default_as_true",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub hub_restrictions: bool,
    #[serde(
        default = "default_as_true",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub exclude_watched: bool,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
    #[serde(
        default = "default_as_true",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub cache_rows: bool,
    #[deprecated]
    #[serde(
        default = "default_as_true",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub cache_rows_refresh: bool,
    #[serde(default, deserialize_with = "deserialize_comma_seperated_string")]
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
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub disable_continue_watching: bool,
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub disable_user_state: bool,
    #[serde(
        default = "default_as_true",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub disable_leaf_count: bool,
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub redirect_streams: bool,
    pub redirect_streams_host: Option<String>,
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub disable_related: bool,
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub disable_transcode: bool,
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub force_maximum_quality: bool,
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub auto_select_version: bool,
    #[serde(default, deserialize_with = "deserialize_comma_seperated_string")]
    pub video_transcode_fallback_for: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_comma_seperated_string")]
    pub force_direct_play_for: Option<Vec<String>>,
    pub test_script: Option<String>,
    #[serde(
        default = "default_as_false",
        deserialize_with = "figment::util::bool_from_str_or_int"
    )]
    pub ntf_watchlist_force: bool,
    #[serde(default, deserialize_with = "deserialize_comma_seperated_string")]
    pub custom_sorting: Option<Vec<String>>,
}

fn default_cache_ttl() -> u64 {
    30 * 60 // 30 minutes
}

pub(crate) fn deserialize_host<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match Deserialize::deserialize(deserializer)? {
        Some::<String>(mut s) => {
            if s.ends_with('/') {
                s.pop();
            }
            Ok(Some(s))
        }
        None => Ok(None),
    }
}

fn default_as_true() -> bool {
    true
}

fn deserialize_hosr() -> bool {
    true
}

impl Config {
    // Note the `nested` option on both `file` providers. This makes each
    // top-level dictionary act as a profile.
    pub fn figment() -> Figment {
        Figment::new().merge(Env::prefixed("REPLEX_"))
    }

    pub fn dynamic(req: &salvo::Request) -> Figment {
        let host = req.headers().get("HOST").unwrap().to_str().unwrap();
        let mut config = Config::figment();
        if host.contains("replex.stream") {
            use data_encoding::BASE32;
            let val: Vec<&str> = host.split(".replex.stream").collect();
            let owned_val = val[0].to_ascii_uppercase().to_owned();
            let mut output = vec![0; BASE32.decode_len(owned_val.len()).unwrap()];
            let len = BASE32.decode_mut(owned_val.as_bytes(), &mut output).unwrap();
            config = config.join(("host", std::str::from_utf8(&output[0 .. len]).unwrap()));
        }
        config
        // Figment::new().merge(Env::prefixed("REPLEX_"))
    }
    // pub fn default() -> Self {
    //     Config { include_watched: false}
    // }
}
