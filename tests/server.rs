use axum::http::StatusCode;
use axum_test_helper::TestClient;
use httpmock::prelude::*;
use pretty_assertions::assert_eq;
extern crate jsonxf;
use replex::config::*;
use replex::proxy::*;
use replex::routes::*;
use replex::url::*;
use rstest::rstest;
use std::env;
use std::fs;
use std::net::SocketAddr;

fn get_mock_server() -> MockServer {
    // let config: Config = Config::figment().extract().unwrap();
    // dbg!(config);
    let mock_server = MockServer::start();
    let _ = mock_server.mock(|when, then| {
        when.method(GET)
            .path("/hubs/sections/6")
            .header("X-Plex-Token", "fakeID")
            .header("X-Plex-Client-Identifier", "fakeID");
        then.status(200)
            .header("content-type", "application/json")
            .body_from_file("tests/mock/in/hubs_sections_6.json");
    });

    let _ = mock_server.mock(|when, then| {
        when.method(GET)
            .path("/library/sections/6/collections")
            .header("X-Plex-Token", "fakeID")
            .header("X-Plex-Client-Identifier", "fakeID");
        then.status(200)
            .header("content-type", "application/json")
            .body_from_file("tests/mock/in/library_sections_6_collections.json");
    });

    let _ = mock_server.mock(|when, then| {
        when.method(GET)
            .path("/library/collections/254688")
            .header("X-Plex-Token", "fakeID")
            .header("X-Plex-Client-Identifier", "fakeID");
        then.status(200)
            .header("content-type", "application/json")
            .body_from_file("tests/mock/in/library_collections_254688.json");
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
            .body_from_file("tests/mock/in/hubs_promoted_6_7.json");
    });

    return mock_server;
}

#[rstest]
#[case::hubs_sections("/hubs/sections/6", "tests/mock/out/hubs_sections_6.json")]
#[case::hubs_promoted(
    format!("{}?contentDirectoryID=6&pinnedContentDirectoryID=6,7", PLEX_HUBS_PROMOTED), "tests/mock/out/hubs_promoted_6.json")
]
#[tokio::test]
/// TODO: Also unit test xml out (not in as we always use json for that)
async fn test_routes(#[case] path: String, #[case] expected_path: String) {
    let mock_server: MockServer = get_mock_server();
    env::set_var(
        "REPLEX_HOST",
        format!("http://{}", mock_server.address().to_string()),
    );
    // let config: Config = Config::figment().extract().unwrap();
    let proxy = Proxy::default();
    let router = router(proxy);
    let client = TestClient::new(router);
    // let path = format!("http://{}{}", client.addr, path);
    let res = client
        .get(&path)
        .header("X-Plex-Token", "fakeID")
        .header("X-Plex-Client-Identifier", "fakeID")
        .header("Accept", "application/json")
        // .header("Host", "127.0.0.1:80")
        .send()
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let result = res.text().await;
    // let _sup = jsonxf::pretty_print(&result).unwrap();

    let expected =
        fs::read_to_string(&expected_path).expect("Should have been able to read the file");
    // dbg!(&result);
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
