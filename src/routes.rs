use crate::cache::*;
use crate::config::Config;
use crate::logging::*;
use crate::models::*;
use crate::plex_client::*;
use itertools::Itertools;
use crate::transform::*;
use crate::url::*;
use crate::utils::*;
use salvo::cache::{Cache, MemoryStore};
use salvo::compression::Compression;
use salvo::cors::Cors;
use salvo::prelude::*;
use salvo::proxy::Proxy as SalvoProxy;
use std::time::Duration;
use salvo::http::{Mime, Request, Response, StatusCode};
use salvo::http::header::CONTENT_TYPE;


pub fn default_cache() -> Cache<MemoryStore<String>, RequestIssuer> {
    let config: Config = Config::figment().extract().unwrap();
    Cache::new(
        MemoryStore::builder()
            .time_to_live(Duration::from_secs(config.cache_ttl))
            .build(),
        RequestIssuer::default(),
    )
}

pub fn route() -> Router {
    let config: Config = Config::figment().extract().unwrap();
    Router::with_hoop(Cors::permissive().into_handler())
        .hoop(Logger::new())
        .hoop(Timeout::new(Duration::from_secs(60)))
        .hoop(Compression::new().enable_gzip(CompressionLevel::Fastest))
        .get(SalvoProxy::new(config.host.unwrap()))
        .push(
            Router::new()
                .path(PLEX_HUBS_PROMOTED)
                .hoop(default_cache())
                .get(get_hubs_promoted),
                // .get(test),
        )
        .push(
            Router::new()
                .path(format!("{}/<id>", PLEX_HUBS_SECTIONS))
                .hoop(default_cache())
                .get(get_hubs_sections),
        )
        // .push(Router::new().path("/test").get(test))
        .push(Router::new().path("/hello").get(hello))
        .push(
            Router::new()
                .path("/replex/library/collections/<ids>/children")
                .hoop(default_cache())
                .get(get_collections_children),
        )
        // .push(
        //     Router::new()
        //         // .path("/desktop/<**rest>")
        //         // .path("/")
        //         .path("/web/<**rest>")
        //         // .path("/web/index.html")
        //         // .handle(redirect),
        //         .handle(SalvoProxy::new(config.host.unwrap())),
        // )
        .push(
            Router::with_path("<**rest>")
                .handle(redirect),
        )
}

#[handler]
async fn redirect(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let config: Config = Config::figment().extract().unwrap();
    let redirect_url = format!("{}{}", config.host.unwrap(), req.uri_mut().path_and_query().unwrap());
    let mime = mime_guess::from_path(req.uri().path()).first_or_octet_stream();
    dbg!(&mime);
    res.headers_mut().insert(CONTENT_TYPE, mime.as_ref().parse().unwrap());
    res.render("would redirect");
    // res.render(Redirect::temporary(redirect_url));
}

#[handler]
async fn test(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let mut container: MediaContainerWrapper<MediaContainer> =
        MediaContainerWrapper::default();
    container.content_type = get_content_type_from_headers(req.headers_mut());

    res.render(container);
}

#[handler]
async fn hello(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    return res.render("Hello world!");
}

#[handler]
pub async fn get_hubs_promoted(req: &mut Request, res: &mut Response) {
    let params: PlexParams = req.extract().await.unwrap();
    let plex_client = PlexClient::from_request(req, params.clone());
    let content_type = get_content_type_from_headers(req.headers_mut());
    // not sure anymore why i have this lol
    // let content_directory_id_size =
    //     params.clone().content_directory_id.unwrap().len();
    // if content_directory_id_size > usize::try_from(1).unwrap() {
    //     let upstream_res = plex_client.request(req).await.unwrap();
    //     let mut container = from_reqwest_response(upstream_res).await.unwrap();
    //     container.content_type = content_type.clone();
    //     res.render(container);
    // }

    if params.clone().content_directory_id.unwrap()[0]
        != params.clone().pinned_content_directory_id.unwrap()[0]
    {
        // We only fill the first one.
        let mut container: MediaContainerWrapper<MediaContainer> =
            MediaContainerWrapper::default();
        container.content_type = content_type.clone();
        container.media_container.size = Some(0);
        container.media_container.allow_sync = Some(true);
        container.media_container.identifier =
            Some("com.plexapp.plugins.library".to_string());
        return res.render(container);
    }

    // first directory, load everything here because we wanna reemiiiixxx
    add_query_param_salvo(
        req,
        "contentDirectoryID".to_string(),
        params
            .clone()
            .pinned_content_directory_id
            .clone()
            .unwrap()
            .iter()
            .join(",")
            .to_string(),
    );

    // we want guids for banners
    add_query_param_salvo(
        req,
        "includeGuids".to_string(),
        "1".to_string(),
    );

    // Hack, as the list could be smaller when removing watched items. So we request more.
    if let Some(original_count) = params.clone().count {
        add_query_param_salvo(
            req,
            "count".to_string(),
            (original_count * 2).to_string(),
        );
    }

    let upstream_res = plex_client.request(req).await.unwrap();
    let mut container: MediaContainerWrapper<MediaContainer> = match from_reqwest_response(upstream_res).await {
        Ok(r) => r,
        Err(error) => {
            tracing::error!(error = ?error, uri = ?req.uri(), "Failed to get plex response");
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            return;
        }
    };
    container.content_type = content_type;

    TransformBuilder::new(plex_client, params.clone())
        .with_transform(HubStyleTransform)
        .with_transform(HubMixTransform)
        .with_transform(HubChildrenLimitTransform {
            limit: params.clone().count.unwrap(),
        })
        .with_transform(TMDBArtTransform)
        .with_transform(UserStateTransform)
        .with_transform(HubKeyTransform)
        .apply_to(&mut container)
        .await;
    res.render(container);
}

#[handler]
pub async fn get_hubs_sections(req: &mut Request, res: &mut Response) {
    let params: PlexParams = req.extract().await.unwrap();
    let plex_client = PlexClient::from_request(req, params.clone());
    let content_type = get_content_type_from_headers(req.headers_mut());
    // Hack, as the list could be smaller when removing watched items. So we request more.
    if let Some(original_count) = params.clone().count {
        // let count_number: i32 = original_count.parse().unwrap();
        add_query_param_salvo(
            req,
            "count".to_string(),
            (original_count * 2).to_string(),
        );
    }

    // we want guids for banners
    add_query_param_salvo(
        req,
        "includeGuids".to_string(),
        "1".to_string(),
    );

    let upstream_res = plex_client.request(req).await.unwrap();
    let mut container: MediaContainerWrapper<MediaContainer> = match from_reqwest_response(upstream_res).await {
        Ok(r) => r,
        Err(error) => {
            tracing::error!(error = ?error, uri = ?req.uri(), "Failed to get plex response");
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            return;
        }
    };
    container.content_type = content_type;

    TransformBuilder::new(plex_client, params.clone())
        .with_transform(HubSectionDirectoryTransform)
        .with_transform(HubStyleTransform)
        .with_transform(HubChildrenLimitTransform {
            limit: params.clone().count.unwrap(),
        })
        .with_transform(TMDBArtTransform)
        .with_transform(UserStateTransform)
        .with_transform(HubKeyTransform)
        // .with_filter(CollectionHubPermissionFilter)
        .with_filter(WatchedFilter)
        .apply_to(&mut container)
        .await;
    res.render(container); // TODO: FIx XML
}

#[handler]
pub async fn get_collections_children(
    req: &mut Request,
    _depot: &mut Depot,
    res: &mut Response,
) {
    let params: PlexParams = req.extract().await.unwrap();
    let collection_ids = req.param::<String>("ids").unwrap();
    let collection_ids: Vec<u32> = collection_ids
        .split(',')
        .filter(|&v| !v.parse::<u32>().is_err())
        .map(|v| v.parse().unwrap())
        .collect();
    let plex_client = PlexClient::from_request(req, params.clone());
    let content_type = get_content_type_from_headers(req.headers_mut());

    // We dont listen to pagination. We have a hard max of 250 per collection
    let limit = Some(250); // plex its max
    let offset = Some(0);

    // create a stub
    let mut container: MediaContainerWrapper<MediaContainer> =
        MediaContainerWrapper::default();
    container.content_type = content_type;
    let size = container.media_container.children().len();
    container.media_container.size = Some(size.try_into().unwrap());
    container.media_container.offset = offset;

    // filtering of watched happens in the transform
    TransformBuilder::new(plex_client, params.clone())
        .with_transform(LibraryMixTransform {
            collection_ids,
            offset,
            limit,
        })
        .with_transform(TMDBArtTransform)
        .with_transform(UserStateTransform)
        .apply_to(&mut container)
        .await;
    res.render(container); // TODO: FIx XML
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use rstest::rstest;
    use salvo::prelude::*;
    use salvo::test::{ResponseExt, TestClient};
    use std::env;

    #[rstest]
    #[case::hubs_sections(
        "/hubs/sections/6",
        "tests/mock/out/hubs_sections_6.json"
    )]
    #[case::hubs_promoted(
        format!("{}?contentDirectoryID=6&pinnedContentDirectoryID=6,7", PLEX_HUBS_PROMOTED), "tests/mock/out/hubs_promoted_6.json")
    ]
    #[tokio::test]
    async fn test_routes(#[case] path: String, #[case] expected_path: String) {
        let mock_server = get_mock_server();
        env::set_var(
            "REPLEX_HOST",
            format!("http://{}", mock_server.address().to_string()),
        );

        let service = Service::new(super::route());

        let content =
            TestClient::get(format!("http://127.0.0.1:5800/{}", &path))
                .add_header("X-Plex-Token", "fakeID", true)
                .add_header("X-Plex-Client-Identifier", "fakeID", true)
                .add_header("Accept", "application/json", true)
                .send((&service))
                .await
                .take_string()
                .await
                .unwrap();
        assert_eq!(content, "Hello world!");
    }

    #[tokio::test]
    async fn test_hello_world() {
        let mock_server = get_mock_server();
        env::set_var(
            "REPLEX_HOST",
            format!("http://{}", mock_server.address().to_string()),
        );

        let service = Service::new(super::route());

        let content =
            TestClient::get(format!("http://127.0.0.1:5800/{}", "hello"))
                .send((&service))
                .await
                .take_string()
                .await
                .unwrap();
        assert_eq!(content, "Hello world!");
    }
}
