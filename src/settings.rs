#![allow(deprecated)]
use config::Config;
use lazy_static::lazy_static;
use std::error::Error;
use std::sync::RwLock;

lazy_static! {
    pub static ref SETTINGS: RwLock<Config> = RwLock::new(Config::builder()
    .set_default("host", "http://46-4-30-217.01b0839de64b49138531cab1bf32f7c2.plex.direct:42405").unwrap() // TODO: RENAME to plex_host
    // Add in `./Settings.toml`
    // .add_source(config::File::with_name("examples/simple/Settings"))
    // Add in settings from the environment (with a prefix of APP)
    // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
    .add_source(config::Environment::with_prefix("APP"))
    .build()
    .unwrap());
}