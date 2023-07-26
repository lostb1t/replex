use axum::{
    body::Body,
    extract::Path,
    extract::State,
    response::Redirect,
    // http::{uri::Uri, Request, Response},
    routing::get,
    Router,
};
use std::{convert::Infallible, env, net::SocketAddr, time::Duration};
use tracing::instrument;
// use axum::headers::ContentType;

// use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
// use axum_tracing_opentelemetry::middleware::OtelInResponseLayer;
use http::{Request, Response};

// use hyper::{client::HttpConnector, Body};

use crate::config::*;
use crate::models::*;
use crate::plex_client::*;
use crate::proxy::*;
use crate::url::*;
use crate::utils::*;
// use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
use itertools::Itertools;
use tower::ServiceBuilder;
use tower_http::cors::AllowOrigin;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::Registry;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn router(proxy: Proxy) -> Router {
    Router::new()
        .route(PLEX_HUBS_PROMOTED, get(get_hubs_promoted))
        // .route("/hubs/sections/:id/*path", get(default_handler))
        .route(
            &format!("{}/:id", PLEX_HUBS_SECTIONS),
            get(get_hubs_sections),
        )
        .route(
            "/replex/library/collections/:ids/children",
            get(get_collections_children),
        )
        .fallback(default_handler)
        .with_state(proxy)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .layer(CorsLayer::permissive())
        // .layer(OtelInResponseLayer::default())
        // .layer(OtelAxumLayer::default())
        //.layer(
        //    CorsLayer::new().allow_origin(AllowOrigin::mirror_request()), // TODO: Limit to https://app.plex.tv
        //)
}

async fn shutdown_signal() {
    // opentelemetry::global::shutdown_tracer_provider();
}

async fn default_handler(State(proxy): State<Proxy>, req: Request<Body>) -> Response<Body> {
    proxy.request(req).await.unwrap()
}

#[instrument]
#[allow(dead_code)]
async fn redirect_to_host(
    State(_proxy): State<Proxy>,
    req: Request<Body>,
) -> axum::response::Redirect {
    let config: Config = Config::figment().extract().unwrap();
    Redirect::temporary(&config.host)
}

#[instrument]
async fn get_hubs_sections(
    State(proxy): State<Proxy>,
    mut req: Request<Body>,
) -> MediaContainerWrapper<MediaContainer> {
    let plex = PlexClient::from(&req);
    let platform = get_header_or_param("x-plex-platform".to_owned(), &req); 
    // Hack, as the list could be smaller when removing watched items. So we request more.
    let mut options = ReplexOptions::default();
    if let Some(original_count) = get_header_or_param("count".to_owned(), &req) {
        let count_number: i32 = original_count.parse().unwrap();
        req = add_query_param(req, "count", &(count_number * 2).to_string());
        options = ReplexOptions {
            limit: Some(count_number),
            platform
        };
    }

    let resp = proxy.request(req).await.unwrap();
    let container = from_response(resp).await.unwrap();
    container.replex(plex, options).await
}

#[instrument]
async fn get_hubs_promoted(
    State(proxy): State<Proxy>,
    mut req: Request<Body>,
) -> MediaContainerWrapper<MediaContainer> {
    let ids_header = get_header_or_param("contentDirectoryID".to_owned(), &req).unwrap();
    let content_directory_ids: Vec<&str> = ids_header.split(',').collect();
    let platform = get_header_or_param("x-plex-platform".to_owned(), &req); 

    if content_directory_ids.len() > 1 {
        let resp = proxy.request(req).await.unwrap();
        return from_response(resp).await.unwrap();
    }

    let pinned_id_header =
        get_header_or_param("pinnedContentDirectoryID".to_owned(), &req).unwrap();
    let pinned_ids: Vec<&str> = pinned_id_header.split(',').collect();

    if content_directory_ids[0] != pinned_ids[0] {
        // We only fill the first one.
        let mut c: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper::default();
        c.content_type = get_content_type(req);
        c.media_container.size = Some(0);
        c.media_container.allow_sync = Some(true);
        c.media_container.identifier = Some("com.plexapp.plugins.library".to_string());
        return c;
    }

    req = add_query_param(req, "contentDirectoryID", &pinned_id_header);

    // Hack, as the list could be smaller when removing watched items. So we request more.
    let mut options = ReplexOptions::default();
    if let Some(original_count) = get_header_or_param("count".to_owned(), &req) {
        let count_number: i32 = original_count.parse().unwrap();
        req = add_query_param(req, "count", &(count_number * 2).to_string());
        options = ReplexOptions {
            limit: Some(count_number),
            platform: platform,
        };
    }

    let plex = PlexClient::from(&req);
    let resp = proxy.request(req).await.expect("Expected an response");
    let container = from_response(resp).await.unwrap();
    container.replex(plex, options).await
}

#[instrument]
async fn get_collections_children(
    State(_proxy): State<Proxy>,
    Path(ids): Path<String>,
    req: Request<Body>,
) -> MediaContainerWrapper<MediaContainer> {
    let collection_ids: Vec<u32> = ids.split(',').map(|v| v.parse().unwrap()).collect();
    let collection_ids_len: i32 = collection_ids.len() as i32;
    let plex = PlexClient::from(&req);
    let mut children: Vec<MetaData> = vec![];
    let reversed: Vec<u32> = collection_ids.iter().copied().rev().collect();
    let platform = get_header_or_param("x-plex-platform".to_owned(), &req); 
    let mut offset: Option<i32> = None;
    let mut original_offset: Option<i32> = None;
    if let Some(i) = get_header_or_param("X-Plex-Container-Start".to_string(), &req) {
        offset = Some(i.parse().unwrap());
        original_offset = offset;
        offset = Some(offset.unwrap() / collection_ids_len);
    }
    let mut limit: Option<i32> = None;
    let mut original_limit: Option<i32> = None;
    if let Some(i) = get_header_or_param("X-Plex-Container-Size".to_string(), &req) {
        limit = Some(i.parse().unwrap());
        original_limit = limit;
        limit = Some(limit.unwrap() / collection_ids_len);
    }

    // dbg!(&offset);
    let mut total_size: i32 = 0;
    for id in reversed {
        let mut c = plex
            .get_collection_children(id, offset.clone(), limit.clone())
            .await
            .unwrap();
        total_size += c.media_container.total_size.unwrap();
        // dbg!(c.media_container.total_size);
        // dbg!(c.media_container.children().len());
        match children.is_empty() {
            false => {
                children = children
                    .into_iter()
                    .interleave(c.media_container.children())
                    .collect::<Vec<MetaData>>();
            }
            true => children.append(&mut c.media_container.children()),
        }
    }

    let mut container: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper::default();
    container.content_type = get_content_type_from_headers(req.headers());
    // so not change the child type, metadata is needed for collections
    container.media_container.metadata = children;
    let size = container.media_container.children().len();
    container.media_container.size = Some(size.try_into().unwrap());
    container.media_container.total_size = Some(total_size);
    container.media_container.offset = original_offset.clone();

    let options = ReplexOptions {
        limit: original_limit,
        platform
    };
    container.replex(plex, options).await
}
