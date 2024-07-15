use crate::{
    config::Config,
    models::*,
    plex_client::{self, PlexClient},
    utils::*,
};
use super::MediaStyleTransform;
use super::Transform;
use super::hero_meta;
use async_trait::async_trait;
use futures_util::{
    future::{self, join_all, LocalBoxFuture},
    stream::{FuturesOrdered, FuturesUnordered},
    StreamExt,
};

#[derive(Default, Debug)]
pub struct HubStyleTransform {
    pub is_home: bool, // use clip instead of hero for android
}

pub struct ClientHeroStyle {
    pub enabled: bool,
    pub r#type: String,
    pub style: Option<String>,
    pub child_type: Option<String>,
    pub cover_art_as_thumb: bool, // if we should return the coverart in the thumb field
    pub cover_art_as_art: bool, // if we should return the coverart in the art field
}

impl Default for ClientHeroStyle {
    fn default() -> Self {
        Self {
            enabled: true,
            style: Some("hero".to_string()),
            r#type: "mixed".to_string(),
            child_type: None,
            cover_art_as_thumb: true,
            cover_art_as_art: false,
        }
    }
}

#[derive(Debug)]
pub enum DeviceType {
    Tv,
    Mobile,
}

impl DeviceType {
    pub fn from_product(product: String) -> DeviceType {
        match product.to_lowercase() {
            x if x.contains("(tv)") => DeviceType::Tv,
            _ => DeviceType::Mobile,
        }
    }
}

impl ClientHeroStyle {
    pub fn from_context(context: PlexContext) -> Self {
        // pub fn android(product: String, platform_version: String) -> Self {
        let product = context.product.clone().unwrap_or_default();
        let device_type = DeviceType::from_product(product);
        let platform = context.platform.clone();
        let platform_version =
            context.platform_version.clone().unwrap_or_default();

        match platform {
            Platform::Android => {
                match device_type {
                    DeviceType::Tv => {
                    //   dbg!(context);
                      Self {
                          style: Some("hero".to_string()),
                          // clip wil make the item info disappear on TV
                          r#type: "clip".to_string(),
                          // using clip makes it load thumbs instead of art as cover art. So we don't have to touch the background
                          child_type: Some("clip".to_string()),
                          cover_art_as_art: true, // Home doesn't work correctly without.
                          cover_art_as_thumb: true,
                          ..ClientHeroStyle::default()
                      }
                    }
                    _ => Self {
                        style: None,
                        r#type: "clip".to_string(),
                        child_type: Some("clip".to_string()),
                        cover_art_as_art: true,
                        ..ClientHeroStyle::default()
                    },
                }
            }
            Platform::Roku => ClientHeroStyle::roku(),
            Platform::Ios => ClientHeroStyle::ios_style(),
            Platform::TvOS => ClientHeroStyle::tvos_style(),
            _ => {
              ClientHeroStyle::default()
          }
            // _ => {
            //     if product.starts_with("Plex HTPC") {
            //         ClientHeroStyle::htpc_style()
            //     } else {
            //         match product.to_lowercase().as_ref() {
            //             "plex for lg" => ClientHeroStyle::htpc_style(),
            //             "plex for xbox" => ClientHeroStyle::htpc_style(),
            //             "plex for ps4" => ClientHeroStyle::htpc_style(),
            //             "plex for ps5" => ClientHeroStyle::htpc_style(),
            //             "plex for ios" => ClientHeroStyle::ios_style(),
            //             _ => ClientHeroStyle::default(),
            //         }
            //     }
            // }
        }
    }

    pub fn roku() -> Self {
        Self {
            style: Some("hero".to_string()),
            ..ClientHeroStyle::default()
        }
    }

    pub fn htpc_style() -> Self {
        Self {
            ..ClientHeroStyle::default()
        }
    }

    pub fn ios_style() -> Self {
        Self {
            cover_art_as_art: true,
            cover_art_as_thumb: false, // ios doesnt load the subview as hero.
            ..ClientHeroStyle::default()
        }
    }

    pub fn tvos_style() -> Self {
      Self {
          cover_art_as_art: true,
          cover_art_as_thumb: false, // ios doesnt load the subview as hero.
          ..ClientHeroStyle::default()
      }
  }

    // pub fn for_client(platform: Platform, product: String, platform_version: String) -> Self {
    //     match platform {
    //         Platform::Android => PlatformHeroStyle::android(product, platform_version),
    //         Platform::Roku => PlatformHeroStyle::roku(product),
    //         _ => {
    //             if product.starts_with("Plex HTPC") {
    //               ClientHeroStyle::htpc_style()
    //             } else {
    //                 match product.to_lowercase().as_ref() {
    //                     "plex for lg" => ClientHeroStyle::htpc_style(),
    //                     "plex for xbox" => ClientHeroStyle::htpc_style(),
    //                     "plex for ps4" => ClientHeroStyle::htpc_style(),
    //                     "plex for ps5" => ClientHeroStyle::htpc_style(),
    //                     "plex for ios" => ClientHeroStyle::ios_style(),
    //                     _ => ClientHeroStyle::default(),
    //                 }
    //             }
    //         }
    //     }
    // }
}

#[async_trait]
impl Transform for HubStyleTransform {
    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexContext,
    ) {
        let config: Config = Config::figment().extract().unwrap();
        let style = item.style.clone().unwrap_or("".to_string()).to_owned();

        if item.is_hub() {
            // TODO: Check why tries to load non existing collectiin? my guess is no access
            let is_hero =
                item.is_hero(plex_client.clone()).await.unwrap_or(false);
            
            if is_hero {
                let mut style = ClientHeroStyle::from_context(options.clone());

                item.style = style.style;

                item.r#type = style.r#type;
                item.meta = Some(hero_meta());

                let mut futures = FuturesOrdered::new();
                // let now = Instant::now();

                for mut child in item.children() {
                    if style.child_type.clone().is_some() {
                        child.r#type = style.child_type.clone().unwrap();
                    }

                    let client = plex_client.clone();
                    let _options = options.clone();
                    futures.push_back(async move {
                        let mut c = child.clone();
                        let transform =
                            MediaStyleTransform { style: Style::Hero };
                        transform
                            .transform_metadata(&mut c, client, _options)
                            .await;
                        c
                    });
                }
                let children: Vec<MetaData> = futures.collect().await;
                item.set_children(children);
            }
        }
    }
}