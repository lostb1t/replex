use crate::{
    models::*,
    plex_client::{PlexClient},
};
use super::Transform;
use super::ClientHeroStyle;
use super::hero_meta;
use async_trait::async_trait;

pub struct MediaStyleTransform {
    pub style: Style,
}

#[async_trait]
impl Transform for MediaStyleTransform {
    async fn transform_mediacontainer(
        &self,
        mut item: MediaContainer,
        plex_client: PlexClient,
        options: PlexContext,
    ) -> MediaContainer {
        if self.style == Style::Hero {
            item.meta = Some(hero_meta());
        }
        item
    }

    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexContext,
    ) {
        if self.style == Style::Hero {
            let style_def = ClientHeroStyle::from_context(options.clone());
            if style_def.child_type.clone().is_some() {
                item.r#type = style_def.child_type.clone().unwrap();
            }

            let mut guid = item.guid.clone().unwrap();
            if guid.starts_with("plex://episode") && item.parent_guid.is_some() {    
                guid = item.parent_guid.clone().unwrap();
            }
            guid = guid.replace("plex://", "");

            let cover_art = Some(format!("/replex/image/hero/{}?X-Plex-Token={}", 
            // let cover_art = Some(format!("{}://{}/replex/image/hero/{}?X-Plex-Token={}", 
                // match options.forwarded_proto {
                //     Some(v) => v,
                //     None => "http".to_string()
                // },
                // match options.forwarded_host {
                //     Some(v) => v,
                //     None => options.host.clone().unwrap()
                // },
                // options.host.clone().unwrap(), 
                guid,
                options.token.clone().unwrap()
            ));
            //dbg!(&cover_art);
            if cover_art.is_some() {
                // c.art = art.clone();
                item.images = vec![Image {
                    r#type: "coverArt".to_string(),
                    url: cover_art.clone().unwrap(),
                    alt: Some(item.title.clone()),
                }];
                // lots of clients dont listen to the above
                if style_def.cover_art_as_art {
                    item.art = cover_art.clone();
                }

                if style_def.cover_art_as_thumb {
                    item.thumb = cover_art.clone();
                }
            }
        }
        // item
    }
}