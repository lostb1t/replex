use http::{uri::PathAndQuery, Uri};
use plex_api::{HttpClient, HttpClientBuilder};
use serde::{Deserialize, Serialize};
// use serde_json::Result;
use serde_xml_rs::from_str as from_xml_str;

// use simple_proxy::middlewares::Logger;
// use simple_proxy::{Environment, SimpleProxy};

// // use crate::middleware::Cors;
// use plex_proxy::middleware::Cors;



// type Metadata struct {
// 	RatingKey             string       `json:"ratingKey"`
// 	Key                   string       `json:"key"`
// 	GUID                  string       `json:"guid"`
// 	AltGUIDs              []AltGUID    `json:"Guid,omitempty"`
// 	Studio                string       `json:"studio"`
// 	Type                  string       `json:"type"`
// 	Title                 string       `json:"title"`
// 	LibrarySectionTitle   string       `json:"librarySectionTitle"`
// 	LibrarySectionID      int          `json:"librarySectionID"`
// 	LibrarySectionKey     string       `json:"librarySectionKey"`
// 	OriginalTitle         string       `json:"originalTitle,omitempty"`
// 	ContentRating         string       `json:"contentRating"`
// 	Rating                float64      `json:"rating"`
// 	Ratings               []Rating     `json:"Rating,omitempty"`
// 	AudienceRating        float64      `json:"audienceRating"`
// 	Year                  int          `json:"year"`
// 	Tagline               string       `json:"tagline"`
// 	Thumb                 string       `json:"thumb"`
// 	Art                   string       `json:"art"`
// 	Duration              int          `json:"duration"`
// 	OriginallyAvailableAt string       `json:"originallyAvailableAt"`
// 	AddedAt               int          `json:"addedAt"`
// 	UpdatedAt             int          `json:"updatedAt"`
// 	AudienceRatingImage   string       `json:"audienceRatingImage"`
// 	ChapterSource         string       `json:"chapterSource,omitempty"`
// 	Media                 []Media      `json:"Media"`
// 	Genre                 []Genre      `json:"Genre"`
// 	Director              []Director   `json:"Director"`
// 	Writer                []Writer     `json:"Writer"`
// 	Country               []Country    `json:"Country"`
// 	Collection            []Collection `json:"Collection"`
// 	Role                  []Role       `json:"Role"`
// 	PrimaryExtraKey       string       `json:"primaryExtraKey,omitempty"`
// 	TitleSort             string       `json:"titleSort,omitempty"`
// }


#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct MetaData {
    rating_key: String,
    key: String,
    guid: String,
    r#type: String,
    title: String,
    thumb: String,
    art: Option<String>,
    year: Option<i32>,
    promoted: Option<bool>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct Hub {
    key: String,
    hub_key: Option<String>,
    title: String,
    hub_identifier: String,
    context: String,
    r#type: String,
    size: i32,
    more: bool,
    style: String,
    promoted: Option<bool>,
    #[serde(rename = "Metadata")]
    metadata: Option<Vec<MetaData>>,
}


#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct MediaContainer {
    pub size: u32,
    pub allow_sync: bool,
    pub identifier: Option<String>,
    #[serde(rename = "librarySectionID")]
    pub library_section_id: u32,
    pub library_section_title: String,
    #[serde(rename = "librarySectionUUID")]
    pub library_section_uuid: String,
    #[serde(rename = "Hub")]
    pub hub: Option<Vec<Hub>>,
    #[serde(rename = "Metadata")]
    metadata: Option<Vec<MetaData>>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
#[serde(rename_all = "camelCase")]
pub struct MediaContainerWrapper<T> {
    #[serde(rename = "MediaContainer")]
    pub media_container: T,
}

// impl MediaContainerWrapper<MediaContainer> {

// }

// impl Json for MediaContainerWrapper<MediaContainer> {
// }

// impl Point {
//     fn as_json(&self)-> String {
//         return serde_json::to_string(&self).unwrap()
//     }

//     fn from_json(s: &str)-> Self {
//         return serde_json::from_str(s).unwrap()
//     }
// }

// #[derive(Debug, Deserialize, Clone)]
// #[cfg_attr(feature = "tests_deny_unknown_fields", serde(deny_unknown_fields))]
// #[serde(rename_all = "camelCase")]
// pub struct MediaContainerWrapper<T> {
//     #[serde(rename = "MediaContainer")]
//     pub media_container: T,
// }

struct PlexHttpClient {
    pub api_url: String,
    pub x_plex_client_identifier: String,
    pub x_plex_token: String,
}

impl PlexHttpClient {
    fn get(path: String) -> () {

        //let json: MediaContainerWrapper<MediaContainer> = reqwest::get("http://httpbin.org/ip")?.json()?;
    }

    // pub fn set_api_url(self, api_url: String) -> Self
    // {
    //     Self {
    //         client: self.client.and_then(move |mut client| {
    //             client.api_url = Uri::try_from(api_url).map_err(Into::into)?;
    //             Ok(client)
    //         }),
    //     }
    // }
}

async fn get_collections() -> Vec<MetaData> {
    let client = HttpClientBuilder::default()
        .set_api_url("https://plex.sjoerdarendsen.dev")
        .set_x_plex_token("RrZN1WRwYYfao2cuiHs5".to_owned())
        .set_x_plex_client_identifier("etz23lqlxhsdinb7hv9uiu38".to_owned())
        .build()
        .expect("wut went wrong");

    // let server = Server::new("https://plex.sjoerdarendsen.dev", client)
    //     .await
    //     .unwrap();
    // let libraries = server.libraries();
    // let library = if let Library::Movie(lib) = libraries.get(0).unwrap() {
    //     lib
    // } else {
    //     panic!("Unexpected library type");
    // };
    // let collections = library.collections().await.unwrap();

    let movie_collection_container: MediaContainerWrapper<MediaContainer> = client
        .get("/library/sections/1/collections")
        .json()
        .await
        .unwrap();
    let show_collection_container: MediaContainerWrapper<MediaContainer> = client
        .get("/library/sections/3/collections")
        .json()
        .await
        .unwrap();

    let collections = [
        show_collection_container.media_container.metadata.unwrap(),
        movie_collection_container.media_container.metadata.unwrap(),
    ]
    .concat();
    // println!("{:#?}", collections);

    collections
}

// #[derive(Debug, Clone)]
// pub struct Config();
// #[tokio::main]
// async fn main() {
//     // let args = Cli::from_args();

//     let mut proxy = SimpleProxy::new(3005, Environment::Development);
//     let logger = Logger::new();
//     let cors = Cors::new();
//     // let router = Router::new(&Config());

//     // Order matters
//     // proxy.add_middleware(Box::new(router));
//     proxy.add_middleware(Box::new(logger));
//     proxy.add_middleware(Box::new(cors));

//     // Start proxy
//     let _ = proxy.run().await;
// }

#[tokio::main]
async fn main() {
    let string = std::fs::read_to_string("test/hubs.json").unwrap();
    let response = rebuild_hubs_req(string, "json".to_owned()).await;
}

async fn rebuild_hubs_req(s: String, application_type: String) -> String {
    // println!("Hello, world!");

    // let result = match application_type.as_str() {
    //     // Match a single value
    //     "json" => serde_json::from_str(&s),
    //     // Match several values
    //     "xml" => serde_json::from_str(&s),
    //     _ => Ok(Err("err")),
    // };

    let mut result: MediaContainerWrapper<MediaContainer> = serde_json::from_str(&s).unwrap();
    if result.media_container.hub.is_none() { 
        // nothing todo
        return s
    }

    let hub_collections = result.media_container.hub.unwrap();

    let allowed_collections = get_collections().await;


    let allowed_collection_keys: Vec<String> = allowed_collections
        .iter()
        .map(|c| c.key.clone())
        .collect();

    let new_collections: Vec<Hub> = hub_collections
        .into_iter()
        .filter(|c| allowed_collection_keys.contains(&c.key))
        .collect();

    // let allowed_collection_keys: Vec<_> = allowed_collections
    //     .iter()
    //     .map(|c| String::from(c.key.clone()))
    //     .collect();

    // let new_collections: Vec<MetaData> = hub_collections
    //     .into_iter()
    //     .filter(|c| allowed_collection_keys.contains(&c.key))
    //     .collect();
    

    println!("{:#?}", new_collections.len());
    result.media_container.hub = Some(new_collections);
    // println!("{:#?}", collection_keys);
    //serde_json::from_str(&json_string).unwrap();
    // let remotes = response.json::<serde_json::Value>().await?;
    "".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(test)]
    use pretty_assertions::assert_eq;

    #[test]
    fn generic_test() {
        let json_string = std::fs::read_to_string("test/hubs.json").unwrap();
        let result: MediaContainerWrapper<MediaContainer> =
            serde_json::from_str(&json_string).unwrap();
        // println!("{:?}", result);
        println!("{:#?}", result);
        let entry: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper {
            media_container: MediaContainer {
                size: 11,
                identifier: Some("com.plexapp.plugins.library".to_owned()),
                library_section_id: 1,
                hub: Some(vec![]),
                metadata: Some(vec![]),
            },
        };
        assert_eq!(entry, result);
    }

    #[test]
    fn xml_test() {
        let xml_string = std::fs::read_to_string("test/hubs.xml").unwrap();
        let result: MediaContainerWrapper<MediaContainer> = from_xml_str(&xml_string).unwrap();
        // println!("{:?}", result);
        println!("{:#?}", result);
        let entry: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper {
            media_container: MediaContainer {
                size: 11,
                identifier: Some("com.plexapp.plugins.library".to_owned()),
                library_section_id: 1,
                hub: Some(vec![]),
                metadata: Some(vec![]),
            },
        };
        assert_eq!(entry, result);
    }
}
