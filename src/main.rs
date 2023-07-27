#[macro_use]
extern crate tracing;

use itertools::Itertools;
use replex::config::Config;
use replex::models::*;
use replex::plex_client::*;

use replex::url::*;
use replex::utils::*;
// use replex::response::Json;


// use salvo::extract;


use salvo::prelude::*;
use salvo::proxy::Proxy as SalvoProxy;




// use tracing::log::Level;


use tracing::{Level};
use tracing_subscriber;

// #[handler]
// async fn set_plex_proxy(_req: &mut Request, depot: &mut Depot) {
//     let plex_proxy = PlexProxy::new("http://46.4.30.217:42405");
//     // let plex_proxy = Proxy::default();
//     depot.insert("plex_proxy", plex_proxy);
// }

#[tokio::main]
async fn main() {
    // tracing_subscriber::fmt().init();
    tracing_subscriber::fmt()
        .compact()
        .with_line_number(true)
        .with_max_level(Level::INFO)
        .init();

    let config: Config = Config::figment().extract().unwrap();
    let router = Router::new()
        .push(
            Router::new()
                // .hoop(set_plex_proxy)
                .path(PLEX_HUBS_PROMOTED)
                .get(get_hubs_promoted),
        )
        .push(
            Router::new()
                // .hoop(set_plex_proxy)
                .path(format!("{}/<id>", PLEX_HUBS_SECTIONS))
                .get(get_hubs_sections),
        )
        .push(
            Router::new()
                // .hoop(set_plex_proxy)
                .path("/replex/library/collections/<ids>/children")
                .get(get_collections_children),
        )
        // .push(
        //     Router::new()
        //         .path("/<id>/websockets/<**rest>")
        //         .handle(connect),
        // )
        .push(
            Router::new()
                .path("/<id>/websockets/<**rest>")
                .handle(Proxy::new(format!("{}/:/websockets", config.host))),
        )
        .push(Router::with_path("<**rest>").handle(SalvoProxy::new(config.host)));

    let acceptor = TcpListener::new("0.0.0.0:80").bind().await;
    Server::new(acceptor).serve(router).await;
}


#[handler]
async fn connect(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    WebSocketUpgrade::new()
        .upgrade(req, res, |mut ws| async move {
            while let Some(msg) = ws.recv().await {
                let msg = if let Ok(msg) = msg {
                    dbg!(&msg);
                    msg
                } else {
                    // client disconnected
                    return;
                };

                if ws.send(msg).await.is_err() {
                    // client disconnected
                    return;
                }
            }
        })
        .await
}

#[handler]
async fn get_hubs_promoted(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let params: PlexParams = req.extract().await.unwrap();
    // dbg!(&params);
    let plex_client = PlexClient::new(req, params.clone());

    // not sure anymore why i have this lol
    let content_directory_id_size = params.clone().content_directory_id.unwrap().len();
    if content_directory_id_size > usize::try_from(1).unwrap() {
        let upstream_res = plex_client.request(req).await;
        let container = from_response(upstream_res).await.unwrap();
        res.render(container);
    }

    if params.clone().content_directory_id.unwrap()[0]
        != params.clone().pinned_content_directory_id.unwrap()[0]
    {
        // We only fill the first one.
        let mut container: MediaContainerWrapper<MediaContainer> = MediaContainerWrapper::default();
        container.content_type = get_content_type_from_headers(req.headers_mut());
        container.media_container.size = Some(0);
        container.media_container.allow_sync = Some(true);
        container.media_container.identifier = Some("com.plexapp.plugins.library".to_string());
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

    // Hack, as the list could be smaller when removing watched items. So we request more.
    let mut options = ReplexOptions::default();
    if let Some(original_count) = params.clone().count {
        // let count_number: i32 = original_count.parse().unwrap();
        add_query_param_salvo(req, "count".to_string(), (original_count * 2).to_string());
        options = ReplexOptions {
            limit: Some(original_count),
            platform: params.clone().platform,
        };
    }

    let upstream_res: Response = plex_client.request(req).await;
    let mut container: MediaContainerWrapper<MediaContainer> = from_response(upstream_res).await.unwrap();
    container = container.replex(plex_client, options).await;
    res.render(container); // TODO: FIx XML
}

#[handler]
async fn get_hubs_sections(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let params: PlexParams = req.extract().await.unwrap();
    let plex_client = PlexClient::new(req, params.clone());

    // Hack, as the list could be smaller when removing watched items. So we request more.
    let mut options = ReplexOptions::default();
    if let Some(original_count) = params.clone().count {
        // let count_number: i32 = original_count.parse().unwrap();
        add_query_param_salvo(req, "count".to_string(), (original_count * 2).to_string());
        options = ReplexOptions {
            limit: Some(original_count),
            platform: params.clone().platform,
        };
    }

    let upstream_res: Response = plex_client.request(req).await;
    let mut container: MediaContainerWrapper<MediaContainer> = from_response(upstream_res).await.unwrap();
    container = container.replex(plex_client, options).await;
    res.render(container); // TODO: FIx XML
}

#[handler]
async fn get_collections_children(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let params: PlexParams = req.extract().await.unwrap();
    let collection_ids = req.param::<String>("ids").unwrap();
    let collection_ids: Vec<u32> = collection_ids
        .split(',')
        .map(|v| v.parse().unwrap())
        .collect();
    let collection_ids_len: i32 = collection_ids.len() as i32;
    let plex_client = PlexClient::new(req, params.clone());
    let mut children: Vec<MetaData> = vec![];
    let reversed: Vec<u32> = collection_ids.iter().copied().rev().collect();

    let mut offset: Option<i32> = None;
    let mut original_offset: Option<i32> = None;
    if let Some(i) = params.clone().x_plex_container_start {
        offset = Some(i);
        original_offset = offset;
        offset = Some(offset.unwrap() / collection_ids_len);
    }
    let mut limit: Option<i32> = None;
    let mut original_limit: Option<i32> = None;
    if let Some(i) = params.clone().x_plex_container_size {
        limit = Some(i);
        original_limit = limit;
        limit = Some(limit.unwrap() / collection_ids_len);
    }

    // dbg!(&offset);
    let mut total_size: i32 = 0;
    for id in reversed {
        let mut c = plex_client
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
    container.content_type = get_content_type_from_headers(req.headers_mut());

    // so not change the child type, metadata is needed for collections
    container.media_container.metadata = children;
    let size = container.media_container.children().len();
    container.media_container.size = Some(size.try_into().unwrap());
    container.media_container.total_size = Some(total_size);
    container.media_container.offset = original_offset.clone();

    let options = ReplexOptions {
        limit: original_limit,
        platform: params.clone().platform,
    };
    container = container.replex(plex_client, options).await;
    res.render(container); // TODO: FIx XML
}
