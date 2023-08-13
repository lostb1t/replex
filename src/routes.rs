use std::sync::Arc;

use crate::cache::*;
use crate::config::Config;
use crate::logging::*;
use crate::models::*;
use crate::plex_client::*;
use crate::proxy::Proxy;
use crate::timeout::*;
use crate::transform::*;
use crate::url::*;
use crate::utils::*;
use itertools::Itertools;
use salvo::cache::{Cache, MemoryStore};
use salvo::compression::Compression;
use salvo::cors::Cors;
use salvo::http::header::CONTENT_TYPE;
use salvo::http::{Mime, Request, Response, StatusCode};
use salvo::prelude::*;
// use salvo::proxy::Proxy as SalvoProxy;
use salvo::routing::PathFilter;
// use std::time::Duration;
use tokio::time::{sleep, Duration};

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

    // cant use colon in paths. So we do it with an regex
    let guid = regex::Regex::new(":").unwrap();
    PathFilter::register_wisp_regex("colon", guid);

    let proxy = Proxy::with_client(
        config.host.clone().unwrap(),
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap(),
    );

    let mut router = Router::with_hoop(Cors::permissive().into_handler())
        .hoop(Logger::new())
        .hoop(Timeout::new(Duration::from_secs(30)))
        .hoop(Compression::new().enable_gzip(CompressionLevel::Fastest))
        // .hoop(max_concurrency(500))
        .hoop(affix::insert("proxy", Arc::new(proxy.clone())));

    if config.redirect_streams {
        router = router
            .push(
                Router::with_path("/video/<colon:colon>/transcode/<**rest>")
                    .handle(redirect_stream),
            )
            .push(
                Router::with_path("/photo/<colon:colon>/transcode")
                    .hoop(fix_photo_transcode_request)
                    .handle(redirect_stream),
            )
            .push(
                Router::with_path("/<colon:colon>/timeline<**rest>")
                    .handle(redirect_stream),
            )
            //.push(
            //    Router::with_path("/statistics/<**rest>")
            //        .handle(redirect_stream),
            //)
            .push(
                Router::with_path(
                    "/library/parts/<itemid>/<partid>/file.<extension>",
                )
                .handle(redirect_stream),
            );
    }

    // TODO: We could just make a gobal middleware that checks every request for the includeRelated.
    // Not sure of the performance impact tho
    if config.disable_related {
        router = router
            .push(
                Router::new()
                    .path("/library/metadata/<id>/related")
                    .hoop(Timeout::new(Duration::from_secs(5)))
                    .handle(proxy.clone()),
            )
            .push(
                Router::with_path("/library/metadata/<id>")
                    .hoop(disable_related_query)
                    .handle(proxy.clone()),
            )
            .push(
                Router::with_path("/playQueues")
                    .hoop(disable_related_query)
                    .handle(proxy.clone()),
            );
    }

    router = router
        .push(
            Router::new()
                .path(PLEX_HUBS_PROMOTED)
                .hoop(default_cache())
                .get(get_hubs_promoted),
        )
        .push(
            Router::new()
                .path(format!("{}/<id>", PLEX_HUBS_SECTIONS))
                .hoop(default_cache())
                .get(get_hubs_sections),
        )
        // .push(Router::new().path("/ping").get(PlexProxy::new(config.host.clone().unwrap())))
        .push(Router::new().path("/hello").get(hello))
        .push(
            Router::new()
                .path("/replex/library/collections/<ids>/children")
                .hoop(default_cache())
                .get(get_collections_children),
        )
        .push(
            Router::with_path("/photo/<colon:colon>/transcode")
                .hoop(fix_photo_transcode_request)
                .handle(proxy.clone()),
        )
        .push(Router::with_path("<**rest>").handle(proxy.clone()));

    router
}

#[handler]
async fn redirect_stream(
    req: &mut Request,
    _depot: &mut Depot,
    res: &mut Response,
) {
    let config: Config = Config::figment().extract().unwrap();
    let redirect_url = if config.redirect_streams_url.clone().is_some() {
        format!(
            "{}{}",
            config.redirect_streams_url.clone().unwrap(),
            req.uri_mut().path_and_query().unwrap()
        )
    } else {
        format!(
            "{}{}",
            config.host.unwrap(),
            req.uri_mut().path_and_query().unwrap()
        )
    };
    let mime = mime_guess::from_path(req.uri().path()).first_or_octet_stream();
    res.headers_mut()
        .insert(CONTENT_TYPE, mime.as_ref().parse().unwrap());
    res.render(Redirect::temporary(redirect_url));
}

// Google tv requests some weird thumbnail for hero elements. Let fix that
#[handler]
async fn fix_photo_transcode_request(
    req: &mut Request,
    _depot: &mut Depot,
    res: &mut Response,
) {
    let params: PlexParams = req.extract().await.unwrap();
    if params.size.is_some() && params.clone().size.unwrap().contains('-')
    // (catched things like (medlium-240, large-500),i dont think size paramater orks at all, but who knows
    // && params.platform.is_some()
    // && params.clone().platform.unwrap().to_lowercase() == "android"
    {
        let size: String = params
            .clone()
            .size
            .unwrap()
            .split('-')
            .last()
            .unwrap()
            .parse()
            .unwrap();
        add_query_param_salvo(req, "height".to_string(), size.clone());
        add_query_param_salvo(req, "width".to_string(), size.clone());
        add_query_param_salvo(req, "quality".to_string(), "80".to_string());
    }
}

#[handler]
async fn disable_related_query(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    add_query_param_salvo(req, "includeRelated".to_string(), "0".to_string());
}

#[handler]
pub async fn get_hubs_promoted(
    req: &mut Request,
    res: &mut Response,
) -> Result<(), anyhow::Error> {
    let config: Config = Config::figment().extract().unwrap();
    let params: PlexParams = req.extract().await.unwrap();
    let plex_client = PlexClient::from_request(req, params.clone());
    let content_type = get_content_type_from_headers(req.headers_mut());

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
        res.render(container);
        return Ok(());
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
    add_query_param_salvo(req, "includeGuids".to_string(), "1".to_string());

    // Hack, as the list could be smaller when removing watched items. So we request more.
    if !config.include_watched {
        if let Some(original_count) = params.clone().count {
            add_query_param_salvo(
                req,
                "count".to_string(),
                (original_count * 2).to_string(),
            );
        }
    }

    
    let upstream_res = plex_client.request(req).await?;
    match upstream_res.status() {
        reqwest::StatusCode::OK => (),
        status => {
            tracing::error!(status = ?status, uri = ?req.uri(), "Failed to get plex response");
            dbg!("this is error");
            // res.render("");
            return Err(salvo::http::StatusError::internal_server_error().into());
            //res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            //return Ok()
        }
    };

    let mut container: MediaContainerWrapper<MediaContainer> =
        from_reqwest_response(upstream_res).await?;
    // match from_reqwest_response(upstream_res).await
    // if upstream_res.status() == 500
    // let mut container: MediaContainerWrapper<MediaContainer> =
    //     match from_reqwest_response(upstream_res).await {
    //         Ok(r) => r,
    //         Err(error) => {
    //             tracing::error!(error = ?error, uri = ?req.uri(), "Failed to get plex response");
    //             res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
    //             return Ok(())
    //         }
    //     };
    container.content_type = content_type;

    TransformBuilder::new(plex_client, params.clone())
        .with_transform(HubStyleTransform)
        .with_transform(HubMixTransform)
        // .with_transform(HubChildrenLimitTransform {
        //     limit: params.clone().count.unwrap(),
        // })
        .with_transform(UserStateTransform)
        .with_transform(HubKeyTransform)
        .apply_to(&mut container)
        .await;
    res.render(container);
    Ok(())
}

#[handler]
pub async fn get_hubs_sections(req: &mut Request, res: &mut Response) {
    let config: Config = Config::figment().extract().unwrap();
    let params: PlexParams = req.extract().await.unwrap();
    let plex_client = PlexClient::from_request(req, params.clone());
    let content_type = get_content_type_from_headers(req.headers_mut());

    // Hack, as the list could be smaller when removing watched items. So we request more.
    if !config.include_watched {
        if let Some(original_count) = params.clone().count {
            // let count_number: i32 = original_count.parse().unwrap();
            add_query_param_salvo(
                req,
                "count".to_string(),
                (original_count * 2).to_string(),
            );
        }
    }

    // we want guids for banners
    add_query_param_salvo(req, "includeGuids".to_string(), "1".to_string());

    let upstream_res = plex_client.request(req).await.unwrap();
    let mut container: MediaContainerWrapper<MediaContainer> =
        match from_reqwest_response(upstream_res).await {
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
        // .with_transform(HubChildrenLimitTransform {
        //     limit: params.clone().count.unwrap(),
        // })
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
    let mut limit: i32 = params.container_size.unwrap_or(50);
    let mut offset: i32 = params.container_start.unwrap_or(0);
    // if params.container_start.is_some() {
    //     offset = params.container_start;
    // }

    // create a stub
    let mut container: MediaContainerWrapper<MediaContainer> =
        MediaContainerWrapper::default();
    container.content_type = content_type;
    let size = container.media_container.children().len();
    container.media_container.size = Some(size.try_into().unwrap());
    container.media_container.offset = Some(offset);

    // filtering of watched happens in the transform
    TransformBuilder::new(plex_client, params.clone())
        .with_transform(LibraryMixTransform {
            collection_ids: collection_ids.clone(),
            offset,
            limit,
        })
        .with_transform(CollecionArtTransform {
            collection_ids: collection_ids.clone(),
            hub: params.content_directory_id.is_some() // its a guessing game
                && !params.include_collections
                && !params.include_advanced
                && !params.exclude_all_leaves,
        })
        .with_transform(UserStateTransform)
        .apply_to(&mut container)
        .await;
    res.render(container); // TODO: FIx XML
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
    sleep(Duration::from_secs(2)).await;
    println!("2 have elapsed");
    return res.render("Hello world!");
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
