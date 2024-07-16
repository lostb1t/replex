// use serde_derive::Deserialize;
// use serde_derive::Serialize;
use serde::{Deserialize, Serialize};

pub fn watchlist(payload: Payload) {

}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub event: String,
    pub user: bool,
    pub owner: bool,
    #[serde(rename = "Account")]
    pub account: Account,
    #[serde(rename = "Server")]
    pub server: Server,
    #[serde(rename = "Player")]
    pub player: Player,
    #[serde(rename = "Metadata")]
    pub metadata: Metadata,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: i64,
    pub thumb: String,
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub title: String,
    pub uuid: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub local: bool,
    pub public_address: String,
    pub title: String,
    pub uuid: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub library_section_type: String,
    pub rating_key: String,
    pub key: String,
    pub parent_rating_key: String,
    pub grandparent_rating_key: String,
    pub guid: String,
    #[serde(rename = "librarySectionID")]
    pub library_section_id: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub title: String,
    pub grandparent_key: String,
    pub parent_key: String,
    pub grandparent_title: Option<String>,
    pub parent_title: Option<String>,
    pub summary: String,
    pub index: i64,
    pub parent_index: i64,
    pub rating_count: i64,
    pub thumb: String,
    pub art: String,
    pub parent_thumb: String,
    pub grandparent_thumb: String,
    pub grandparent_art: String,
    pub added_at: i64,
    pub updated_at: i64,
}
