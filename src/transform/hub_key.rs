use crate::{
    models::*,
    plex_client::{PlexClient},
};

use super::Transform;
use async_trait::async_trait;

#[derive(Default, Debug)]
pub struct HubKeyTransform;

/// We point to replex so we can do some transform on the children calls
#[async_trait]
impl Transform for HubKeyTransform {
    async fn transform_metadata(
        &self,
        item: &mut MetaData,
        plex_client: PlexClient,
        options: PlexContext,
    ) {

        if item.is_hub()
            && item.key.is_some()
            && !item.key.clone().unwrap().starts_with("/replex")
        {
            // might already been set by the mixings
            // setting an url argument crashes client. So we use the path
            let old_key = item.key.clone().unwrap();
            item.key = Some(format!(
                "/replex/{}{}",
                item.style
                    .clone()
                    .unwrap_or(Style::Shelf.to_string().to_lowercase()),
                old_key
            ));
            tracing::debug!(old_key = old_key, key = &item.key, "Replacing hub key");
        }

    }
}