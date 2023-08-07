use std::borrow::Cow;

use tmdb_api::tvshow::search::TVShowSearch;
use tmdb_api::prelude::Command;
use tmdb_api::common::image::Image;
use tmdb_api::Client;
use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer, Serialize};

use crate::config::Config;

pub(crate) static TMDB_CLIENT: Lazy<Client> =
    Lazy::new(|| {
        let config: Config = Config::figment().extract().unwrap();
        Client::new(config.tmdb_api_key.unwrap())
    });

#[derive(Clone, Debug, Default)]
pub struct TVShowImages {
    /// ID of the movie
    pub show_id: u64,
    /// ISO 639-1 value to display translated data for the fields that support it.
    pub language: Option<String>,
}

impl TVShowImages {
    pub fn new(show_id: u64) -> Self {
        Self {
            show_id,
            language: None,
        }
    }

    pub fn with_language(mut self, value: Option<String>) -> Self {
        self.language = value;
        self
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TVShowImagesResult {
    pub id: u64,
    pub backdrops: Vec<Image>,
    pub posters: Vec<Image>,
    pub logos: Vec<Image>,
}

impl Command for TVShowImages {
    type Output = TVShowImagesResult;

    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(format!("/tv/{}/images", self.show_id))
    }

    fn params(&self) -> Vec<(&'static str, Cow<'_, str>)> {
        if let Some(ref language) = self.language {
            vec![("language", Cow::Borrowed(language))]
        } else {
            Vec::new()
        }
    }
}