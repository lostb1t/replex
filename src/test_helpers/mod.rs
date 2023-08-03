use httpmock::prelude::*;

// simulate_standalone_server();

pub(crate) fn get_mock_server() -> MockServer {
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

    let _ = mock_server.mock(|when, then| {
        when.method(GET)
            .path("/library/sections");
        then.status(200)
            .header("content-type", "application/json")
            .body_from_file("tests/mock/in/library_sections.json");
    });

    return mock_server;
}