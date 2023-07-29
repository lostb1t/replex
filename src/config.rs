use serde::Deserialize;
use figment::{Figment, providers::{Env}};

fn default_as_false() -> bool {
    false
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Config {
    #[serde( default)]
    pub host: String,
    #[serde( default = "default_as_false")]
    pub include_watched: bool,
    #[serde( default = "default_as_false")]
    pub ssl_enable: bool,
    pub ssl_domain: Option<String>,
}

impl Config {
    // Note the `nested` option on both `file` providers. This makes each
    // top-level dictionary act as a profile.
    pub fn figment() -> Figment {
        Figment::new()
            .merge(Env::prefixed("REPLEX_"))
    }
    // pub fn default() -> Self {
    //     Config { include_watched: false}
    // }

}