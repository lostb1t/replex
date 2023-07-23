#[macro_use]
extern crate tracing;

use std::{convert::Infallible, env, net::SocketAddr, time::Duration};
use axum::{
    body::Body,
    extract::Path,
    extract::State,
    response::Redirect,
    // http::{uri::Uri, Request, Response},
    routing::get,
    Router,
};
// use axum::headers::ContentType;

use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use axum_tracing_opentelemetry::middleware::OtelInResponseLayer;
use http::{Request, Response};

// use hyper::{client::HttpConnector, Body};

use itertools::Itertools;
use replex::models::*;
use replex::plex_client::*;
use replex::proxy::*;
use replex::settings::*;
use replex::url::*;
use replex::utils::*;
use tower_http::cors::AllowOrigin;
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;
use tracing_subscriber::Registry;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
use tower::ServiceBuilder;

#[tokio::main]
async fn main() {
    // set_default_env_var("REPLEX_", "8080");
    // let new_relic_api_key = SETTINGS.read().unwrap().get::<String>("host");
    // env_logger::init();
    // https://github.com/tokio-rs/axum/blob/main/examples/tracing-aka-logging/src/main.rs
    // if let new_relic_api_key = SETTINGS.read().unwrap().get::<String>("newrelic_api_key").unwrap() {
    //     let newrelic = tracing_newrelic::layer(new_relic_api_key);
    //     tracing_subscriber::registry()
    //         .with(newrelic)
    //         .with(tracing_subscriber::fmt::layer())
    //         .init();

    //     // let fmt = tracing_subscriber::fmt::layer();
    //     // let subscriber = Registry::default().with(newrelic).with(fmt).with(target);
    //     // tracing::subscriber::set_global_default(subscriber)
    //     //     .expect("failed to initilize tracing subscriber");
    // } else {
    //     tracing_subscriber::fmt::init();
    // }

    // let content_type = if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
    //     content_type
    // } else {
    //     return false;
    // };

    tracing_subscriber::fmt::init();
    // env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "https://otlp.eu01.nr-data.net");
    //OTEL_EXPORTER_OTLP_TRACES_ENDPOINT
    // init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers().unwrap();

    let proxy = Proxy::default();
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    info!(message = "Listening on", %addr);
    axum::Server::bind(&addr)
        .serve(router(proxy).into_make_service())
        .await
        .unwrap();
}

fn router(proxy: Proxy) -> Router {
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
        // .layer(OtelInResponseLayer::default())
        // .layer(OtelAxumLayer::default())
        .layer(
            CorsLayer::new().allow_origin(AllowOrigin::mirror_request()), // TODO: Limit to https://app.plex.tv
        )
}

async fn shutdown_signal() {
    opentelemetry::global::shutdown_tracer_provider();
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
    //Redirect::to("https://46-4-30-217.01b0839de64b49138531cab1bf32f7c2.plex.direct:42405")
    //proxy.request(req).await.unwrap()
    // debug!("Redirecting: {:?}", &req.uri());
    // debug!("req: {:?}", req);
    Redirect::temporary(&SETTINGS.read().unwrap().get::<String>("host").unwrap())
}

#[instrument]
async fn get_hubs_sections(
    State(proxy): State<Proxy>,
    req: Request<Body>,
) -> MediaContainerWrapper<MediaContainer> {
    let plex = PlexClient::from(&req);
    let resp = proxy.request(req).await.unwrap();

    let mut container = from_response(resp).await.unwrap();
    container = container
        .fix_permissions(&plex)
        .await
        .apply_hub_style(&plex)
        .await
        .clone();
    container
}

#[instrument]
async fn get_hubs_promoted(
    State(proxy): State<Proxy>,
    mut req: Request<Body>,
) -> MediaContainerWrapper<MediaContainer> {
    let ids_header = get_header_or_param("contentDirectoryID".to_owned(), &req).unwrap();
    let content_directory_ids: Vec<&str> = ids_header.split(',').collect();
    // Cant handle multiple directories yet
    if content_directory_ids.len() > 1 {
        let resp = proxy.request(req).await.unwrap();
        return from_response(resp).await.unwrap();
    }

    let pinned_id_header =
        get_header_or_param("pinnedContentDirectoryID".to_owned(), &req).unwrap();
    let pinned_ids: Vec<&str> = pinned_id_header.split(',').collect();

    if content_directory_ids[0] != pinned_ids[0] {
        // We only fill the first one.
        //let content_type = get_content_type(req);
        // debug!("Gonna return an empty response");
        let mut c: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper::default();
        c.content_type = get_content_type(req);
        // c.content_type = ContentType::Json;
        c.media_container.size = Some(0);
        c.media_container.allow_sync = Some(true);
        c.media_container.identifier = Some("com.plexapp.plugins.library".to_string());
        return c;
    }

    // req = remove_param(req, "contentDirectoryID");
    req = add_query_param(req, "contentDirectoryID", &pinned_id_header);

    let plex = PlexClient::from(&req);
    let resp = proxy.request(req).await.expect("Expected an response");
    let mut container = from_response(resp).await.unwrap();
    let remove_watched = &SETTINGS
        .read()
        .unwrap()
        .get::<bool>("include_watched")
        .unwrap();
    if *remove_watched {
        container = container.remove_watched();
    }
    container.process_hubs(plex).await
}

#[instrument]
async fn get_collections_children(
    State(_proxy): State<Proxy>,
    Path(ids): Path<String>,
    req: Request<Body>,
) -> MediaContainerWrapper<MediaContainer> {
    let collection_ids: Vec<u32> = ids.split(',').map(|v| v.parse().unwrap()).collect();
    let plex = PlexClient::from(&req);
    let mut children: Vec<MetaData> = vec![];
    let reversed: Vec<u32> = collection_ids.iter().copied().rev().collect();

    for id in reversed {
        let mut c = plex.get_collection_children(id).await.unwrap();

        // children = [children, c.media_container.children()].concat();
        match children.is_empty() {
            false => {
                children = children
                    .into_iter()
                    .interleave(c.media_container.children())
                    .collect::<Vec<MetaData>>();
            }
            true => children.append(&mut c.media_container.children()),
        }
        children.append(&mut c.media_container.children())
    }

    let mut container: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper::default();
    container.content_type = get_content_type_from_headers(req.headers());
    // so not change the child type, video is needed for collections
    container.media_container.video = children;
    let remove_watched = &SETTINGS
        .read()
        .unwrap()
        .get::<bool>("include_watched")
        .unwrap();
    if *remove_watched {
        container.media_container.remove_watched();
    }
    let size = container.media_container.children().len();
    container.media_container.size = Some(size.try_into().unwrap());
    container.media_container.total_size = Some(size.try_into().unwrap());
    container.media_container.offset = Some(0);

    // container = container.make_mixed();
    container
}

// impl Clone for Request<T> {
//     fn clone(&self) -> Request<Body> {
//         let (mut parts, _) = self.into_parts();
//         Request::from_parts(parts, Body::empty())
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;
    use httpmock::prelude::*;
    use pretty_assertions::assert_eq;
    extern crate jsonxf;
    use rstest::rstest;
    use std::fs;

    fn get_mock_server() -> MockServer {
        let mock_server = MockServer::start();
        let _ = mock_server.mock(|when, then| {
            when.method(GET)
                .path("/hubs/sections/6")
                .header("X-Plex-Token", "fakeID")
                .header("X-Plex-Client-Identifier", "fakeID");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("test/mock/in/hubs_sections_6.json");
        });

        let _ = mock_server.mock(|when, then| {
            when.method(GET)
                .path("/library/sections/6/collections")
                .header("X-Plex-Token", "fakeID")
                .header("X-Plex-Client-Identifier", "fakeID");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("test/mock/in/library_sections_6_collections.json");
        });

        let _ = mock_server.mock(|when, then| {
            when.method(GET)
                .path("/hubs/promoted")
                .header("X-Plex-Token", "fakeID")
                .header("X-Plex-Client-Identifier", "fakeID")
                .query_param("pinnedContentDirectoryID", "6,7")
                .query_param("contentDirectoryID", "6,7");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("test/mock/in/hubs_promoted_6_7.json");
        });

        return mock_server;
    }

    #[rstest]
    #[case::hubs_sections("/hubs/sections/6", "test/mock/out/hubs_sections_6.json")]
    #[case::hubs_promoted(format!("{}?contentDirectoryID=6&pinnedContentDirectoryID=6,7", PLEX_HUBS_PROMOTED), "test/mock/out/hubs_promoted_6.json")]
    #[tokio::test]
    async fn test_routes(#[case] path: String, #[case] expected_path: String) {
        let mock_server: MockServer = get_mock_server();
        // let expected_out = "test/mock/out/hubs_sections_6.json";
        dbg!(&path);
        SETTINGS
            .write()
            .unwrap()
            .set("host", mock_server.base_url())
            .unwrap();

        let proxy = Proxy::default();
        let router = router(proxy);
        // dbg!("everything stup");
        let client = TestClient::new(router);
        let res = client
            .get(&path)
            .header("X-Plex-Token", "fakeID")
            .header("X-Plex-Client-Identifier", "fakeID")
            .header("Accept", "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);

        let result = res.text().await;
        let _sup = jsonxf::pretty_print(&result).unwrap();

        let expected =
            fs::read_to_string(&expected_path).expect("Should have been able to read the file");

        assert_eq!(
            jsonxf::pretty_print(&result).unwrap(),
            jsonxf::pretty_print(&expected).unwrap()
        );
    }

    // #[tokio::test]
    // async fn test_hubs_sections() {
    //     let mock_server: MockServer = get_mock_server();
    //     let expected_out = "test/mock/out/hubs_sections_6.json";

    //     SETTINGS
    //         .write()
    //         .unwrap()
    //         .set("host", mock_server.base_url())
    //         .unwrap();

    //     let proxy = Proxy::default();
    //     let mut router = router(proxy);
    //     // dbg!("everything stup");
    //     let client = TestClient::new(router);
    //     let res = client
    //         .get("/hubs/sections/6")
    //         .header("X-Plex-Token", "fakeID")
    //         .header("X-Plex-Client-Identifier", "fakeID")
    //         .header("Accept", "application/json")
    //         .send()
    //         .await;
    //     assert_eq!(res.status(), StatusCode::OK);

    //     let result = res.text().await;
    //     let sup = jsonxf::pretty_print(&result).unwrap();

    //     let expected =
    //         fs::read_to_string(expected_out).expect("Should have been able to read the file");

    //     assert_eq!(
    //         jsonxf::pretty_print(&result).unwrap(),
    //         jsonxf::pretty_print(&expected).unwrap()
    //     );
    // }
}
