use tmdb_api::tvshow::search::TVShowSearch;
use tmdb_api::prelude::Command;
use tmdb_api::Client;
use once_cell::sync::Lazy;

use crate::config::Config;

pub(crate) static TMDB_CLIENT: Lazy<Client> =
    Lazy::new(|| {
        let config: Config = Config::figment().extract().unwrap();
        Client::new(config.tmdb_api_key.unwrap())
    });