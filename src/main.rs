#[macro_use]
extern crate tracing;
#[macro_use]
extern crate axum_core;

use axum::headers::ContentType as HContentType;
use axum::response::IntoResponse;
use axum::{
    extract::Path,
    extract::State,
    extract::TypedHeader,
    // http::{uri::Uri, Request, Response},
    routing::get,
    routing::put,
    response::Redirect,
    body::Body,
    Router,
};
// use axum::headers::ContentType;
use cached::proc_macro::cached;
// use bytes::Bytes;
// use crate::{
//     body::{Bytes, HttpBody},
//     extract::{rejection::*, FromRequest},
//     BoxError,
// };
use http::{Request, Response};

// use hyper::{client::HttpConnector, Body};

use httplex::models::*;
use httplex::plex_client::*;
use httplex::proxy::*;
use httplex::settings::*;
use httplex::url::*;
use httplex::utils::*;
use itertools::Itertools;
use tower_http::cors::AllowOrigin;
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;

use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    // tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();
    // tokio::spawn(server());
    // let bla = Client::new();
    // let plex_api_client = create_client_from_request()

    // let app = Router::new()
    //     .route("/hubs/promoted", get(get_hubs_promoted))
    //     // .route("/hubs/sections/:id/*path", get(default_handler))
    //     .route("/hubs/sections/:id", get(get_hubs_sections))
    //     .route(
    //         "/hubs/library/collections/:ids/children",
    //         get(get_collections_children),
    //     )
    //     .fallback(default_handler)
    //     // .route("/*path", get(default_handler))
    //     // .route("/*path", put(default_handler))
    //     // .route("/", get(default_handler))
    //     .with_state(proxy)
    //     .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
    //     .layer(
    //         CorsLayer::new().allow_origin(AllowOrigin::mirror_request()), // TODO: Limit to https://app.plex.tv
    //     );
    // let app = App::default();
    let proxy = Proxy::default();
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    // println!("reverse proxy listening on {}", addr);
    info!(message = "Listening on", %addr);
    axum::Server::bind(&addr)
        .serve(router(proxy).into_make_service())
        .await
        .unwrap();
}

fn router(proxy: Proxy) -> Router {
    // let proxy = Proxy {
    //     host: "http://100.91.35.113:32400".to_string(),
    //     client: Client::new(),
    // };

    Router::new()
        .route(PLEX_HUBS_PROMOTED, get(get_hubs_promoted))
        // .route("/hubs/sections/:id/*path", get(default_handler))
        .route(
            &format!("{}/:id", PLEX_HUBS_SECTIONS),
            get(get_hubs_sections),
        )
        .route(
            "/httplex/library/collections/:ids/children",
            get(get_collections_children),
        )
        // .route(
        //     "/video/:placeholder/*wild",
        //     get(redirect_to_host),
        // )
        // .route(
        //     "/photo/:placeholder/*wild",
        //     get(redirect_to_host),
        // )
        // .route(
        //     "/web/static/*wild",
        //     get(redirect_to_host),
        // )
        .fallback(default_handler)
        // .route("/*path", get(default_handler))
        // .route("/*path", put(default_handler))
        // .route("/", get(default_handler))
        .with_state(proxy)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .layer(
            CorsLayer::new().allow_origin(AllowOrigin::mirror_request()), // TODO: Limit to https://app.plex.tv
        )
}

async fn default_handler(State(proxy): State<Proxy>, req: Request<Body>) -> Response<Body> {
    proxy.request(req).await.unwrap()
}

//#[instrument]
async fn redirect_to_host(State(proxy): State<Proxy>, req: Request<Body>) -> axum::response::Redirect {
    //Redirect::to("https://46-4-30-217.01b0839de64b49138531cab1bf32f7c2.plex.direct:42405")
    //proxy.request(req).await.unwrap()
    info!("Redirecting: {:?}", &req.uri());
    // debug!("req: {:?}", req);
    Redirect::temporary(&SETTINGS.read().unwrap().get::<String>("host").unwrap())
}

#[instrument]
async fn get_hubs_sections(
    State(mut proxy): State<Proxy>,
    req: Request<Body>,
) -> MediaContainerWrapper<MediaContainer> {
    // proxy.set_plex_api_from_request(&req).await;
    // let resp = proxy.request(req).await.unwrap();
    // dbg!(&req.content_type);

    let plex = PlexClient::from(&req);
    // dbg!(&plex.content_type);
    let resp = proxy.request(req).await.unwrap();

    // dbg!(&resp.headers());
    // let container =
    //     MediaContainerWrapper::<MediaContainer>::from_response(proxy.request(req).await.unwrap()).unwrap();
    let mut container = from_response(resp).await.unwrap();
    // dbg!(&container);
    // dbg!("YOOOOOO");

    container = container.fix_permissions(plex).await;
    // dbg!("YOOOOOO");

    container
    // MediaContainerWrapper::default()
}

// fn bla(req: Request<Body>) -> String {
//     "hoszaaa".to_string()
// }

// #[cached(    DOESNT WORK BECAUSE OF ARGUMENT State(mut proxy): State<Proxy>
//     time = 720,
//     key = "String",
//     convert = r#"{ bla(&req) }"#
//     // convert = r#"{ format!("{}{}", proxy.host, req.method()) }"#
// )]
async fn get_hubs_promoted(
    State(mut proxy): State<Proxy>,
    mut req: Request<Body>,
) -> MediaContainerWrapper<MediaContainer> {
    let ids_header = get_header_or_param("contentDirectoryID".to_owned(), &req).unwrap();
    let content_directory_ids: Vec<&str> = ids_header.split(',').collect();
    println!("SUP");
    // Cant handle multiple directories yet
    if content_directory_ids.len() > 1 {
        let resp = proxy.request(req).await.unwrap();
        return from_response(resp).await.unwrap();
    }

    let pinned_id_header =
        get_header_or_param("pinnedContentDirectoryID".to_owned(), &req).unwrap();
    let pinned_ids: Vec<&str> = pinned_id_header.split(',').collect();
    // dbg!(pinned_ids);
    //pinnedContentDirectoryID
    // dbg!(&req);
    // debug!("{:#?}", &req);
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
    // dbg!(&req);
    // dbg!(&proxy.host);
    let plex = PlexClient::from(&req);
    let resp = proxy.request(req).await.expect("Expected an response");
    // let (parts, body) = resp.into_parts();
    // dbg!(body_to_string(body).await);
    // return MediaContainerWrapper::default();
    // let mut resp_second = self
    //     .get(format!("/library/sections/{}/collections", id))
    //     .await
    //     .unwrap();
    // let (parts, body) = resp_second.into_parts();
    // dbg!(body_to_string(body).await);
    // dbg!(&req);
    let mut container = from_response(resp).await.unwrap();

    // container.media_container.metadata = vec![];
    // dbg!(&container);
    // container = container.fix_permissions(plex).await;
    container.make_mixed()
}

async fn get_collections_children(
    State(mut proxy): State<Proxy>,
    Path(ids): Path<String>,
    req: Request<Body>,
) -> MediaContainerWrapper<MediaContainer> {
    let collection_ids: Vec<u32> = ids.split(',').map(|v| v.parse().unwrap()).collect();
    let plex = PlexClient::from(&req);
    let mut children: Vec<MetaData> = vec![];
    let reversed: Vec<u32> = collection_ids.iter().copied().rev().collect();
    for id in reversed {
        let mut c = plex.get_collection_children(id).await.unwrap();
        // dbg!(&c.media_container.children());
        //container.media_container.set_children(c.media_container.children()); // TODO: unnecessary. Just need the type (video, directory etc)
        //dbg!(&c.media_container.children());
        match children.is_empty() {
            False => {
                children = children
                    .into_iter()
                    .interleave(c.media_container.children())
                    .collect::<Vec<MetaData>>();
            }
            True => children.append(&mut c.media_container.children()),
        }
        // children.append(&mut c.media_container.children())
    };

    
    // dbg!(req.headers().get("Accept").unwrap());
    let mut container: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper::default();
    container.content_type = get_content_type_from_headers(req.headers());
    // dbg!(&container.content_type);
    let size = children.len();
    // container.media_container.set_children(children);
    container.media_container.directory = children;
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
    use httpmock::{prelude::*, Mock};
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
                .query_param( "pinnedContentDirectoryID", "6,7")
                .query_param( "contentDirectoryID", "6,7");
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
        let mut router = router(proxy);
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
        let sup = jsonxf::pretty_print(&result).unwrap();

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
