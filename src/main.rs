#[macro_use]
extern crate tracing;
// extern crate tracing_subscriber;

use itertools::Itertools;
use opentelemetry::sdk::export::trace::stdout;
use opentelemetry_otlp::WithExportConfig;
use replex::cache::*;
use replex::config::Config;
use replex::logging::*;
use replex::models::*;
use replex::plex_client::*;
use replex::proxy::PlexProxy;
use replex::transform::*;
use replex::url::*;
use replex::utils::*;
use salvo::cache::{Cache, MemoryStore};
use salvo::cors::Cors;
use salvo::prelude::*;
use salvo::proxy::Proxy as SalvoProxy;
use salvo::test::ResponseExt;
use std::time::Duration;
use tonic::metadata::MetadataMap;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

pub fn default_cache() -> Cache<MemoryStore<String>, RequestIssuer>  {
    Cache::new(
        MemoryStore::builder()
            .time_to_live(Duration::from_secs(5))
            .build(),
        RequestIssuer::default(),
    )
}

#[tokio::main]
async fn main() {
    let config: Config = Config::figment().extract().unwrap();
    if config.host.is_none() {
        tracing::error!("REPLEX_HOST is required. Exiting");
        return;
    }

    // TODO: rework this a bit: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/#runtime-configuration-with-layers
    let fmt_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with({
            let otlp;
            if config.newrelic_api_key.is_some() {
                let mut map = MetadataMap::with_capacity(3);
                map.insert(
                    "api-key",
                    config.newrelic_api_key.unwrap().parse().unwrap(),
                );
                let tracer = opentelemetry_otlp::new_pipeline()
                    .tracing()
                    .with_exporter(
                        opentelemetry_otlp::new_exporter()
                            .tonic()
                            .with_tls_config(Default::default())
                            .with_endpoint(
                                "https://otlp.eu01.nr-data.net:443/v1/traces",
                            )
                            .with_metadata(map)
                            .with_timeout(Duration::from_secs(3)),
                    )
                    .install_batch(opentelemetry::runtime::Tokio)
                    .unwrap();
                otlp = tracing_opentelemetry::layer().with_tracer(tracer);
            } else {
                let tracer = stdout::new_pipeline().install_simple();
                otlp = tracing_opentelemetry::layer().with_tracer(tracer);
            }
            otlp
        })
        .with(fmt_layer)
        .init();

    let router = Router::with_hoop(Cors::permissive().into_handler())
        .hoop(Logger::new())
        .hoop(Timeout::new(Duration::from_secs(60)))
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
        .push(Router::new().path("/test").get(test))
        .push(
            Router::new()
                .path("/replex/library/collections/<ids>/children")
                .hoop(default_cache())
                .get(get_collections_children),
        )
        .push(
            Router::with_path("<**rest>")
                .handle(PlexProxy::new(config.host.unwrap())),
        );

    if config.ssl_enable && config.ssl_domain.is_some() {
        let acceptor = TcpListener::new("0.0.0.0:443")
            .acme()
            .cache_path("/data/acme/letsencrypt")
            .add_domain(config.ssl_domain.unwrap())
            .bind()
            .await;
        Server::new(acceptor).serve(router).await;
    } else {
        let acceptor = TcpListener::new("0.0.0.0:80").bind().await;
        Server::new(acceptor).serve(router).await;
    }
}

#[handler]
async fn test(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    return res.render("sup");
}

#[handler]
async fn get_hubs_promoted(req: &mut Request, res: &mut Response) {
    let params: PlexParams = req.extract().await.unwrap();
    let plex_client = PlexClient::new(req, params.clone());

    // not sure anymore why i have this lol
    let content_directory_id_size =
        params.clone().content_directory_id.unwrap().len();
    if content_directory_id_size > usize::try_from(1).unwrap() {
        let upstream_res = plex_client.request(req).await.unwrap();
        let container = from_reqwest_response(upstream_res).await.unwrap();
        res.render(container);
    }

    if params.clone().content_directory_id.unwrap()[0]
        != params.clone().pinned_content_directory_id.unwrap()[0]
    {
        // We only fill the first one.
        let mut container: MediaContainerWrapper<MediaContainer> =
            MediaContainerWrapper::default();
        container.content_type =
            get_content_type_from_headers(req.headers_mut());
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

    // Hack, as the list could be smaller when removing watched items. So we request more.
    if let Some(original_count) = params.clone().count {
        add_query_param_salvo(
            req,
            "count".to_string(),
            (original_count * 2).to_string(),
        );
    }

    let upstream_res = plex_client.request(req).await.unwrap();
    let mut container: MediaContainerWrapper<MediaContainer> =
        from_reqwest_response(upstream_res).await.unwrap();

    TransformBuilder::new(plex_client, params.clone())
        .with_transform(HubStyleTransform)
        .with_transform(HubMixTransform)
        .with_transform(LimitTransform {
            limit: params.clone().count.unwrap(),
        })
        .with_filter(CollectionHubPermissionFilter)
        .with_filter(WatchedFilter)
        .apply_to(&mut container)
        .await;
    res.render(container);
}

#[handler]
async fn get_hubs_sections(req: &mut Request, res: &mut Response) {
    let params: PlexParams = req.extract().await.unwrap();
    let plex_client = PlexClient::new(req, params.clone());

    // Hack, as the list could be smaller when removing watched items. So we request more.
    if let Some(original_count) = params.clone().count {
        // let count_number: i32 = original_count.parse().unwrap();
        add_query_param_salvo(
            req,
            "count".to_string(),
            (original_count * 2).to_string(),
        );
    }

    let upstream_res = plex_client.request(req).await.unwrap();
    let mut container: MediaContainerWrapper<MediaContainer> =
        from_reqwest_response(upstream_res).await.unwrap();
    TransformBuilder::new(plex_client, params.clone())
        .with_transform(HubStyleTransform)
        .with_transform(LimitTransform {
            limit: params.clone().count.unwrap(),
        })
        .with_filter(CollectionHubPermissionFilter)
        .with_filter(WatchedFilter)
        .apply_to(&mut container)
        .await;
    res.render(container); // TODO: FIx XML
}

#[handler]
async fn get_collections_children(
    req: &mut Request,
    _depot: &mut Depot,
    res: &mut Response,
) {
    let params: PlexParams = req.extract().await.unwrap();
    let collection_ids = req.param::<String>("ids").unwrap();
    let collection_ids: Vec<u32> = collection_ids
        .split(',')
        .map(|v| v.parse().unwrap())
        .collect();
    let collection_ids_len: i32 = collection_ids.len() as i32;
    let plex_client = PlexClient::new(req, params.clone());

    let mut offset: Option<i32> = None;
    let mut original_offset: Option<i32> = None;
    if let Some(i) = params.clone().container_start {
        offset = Some(i);
        original_offset = offset;
        offset = Some(offset.unwrap() / collection_ids_len);
    }
    let mut limit: Option<i32> = None;
    let mut original_limit: Option<i32> = None;
    if let Some(i) = params.clone().container_size {
        limit = Some(i);
        original_limit = limit;
        limit = Some(limit.unwrap() / collection_ids_len);
    }

    // create a stub
    let mut container: MediaContainerWrapper<MediaContainer> =
        MediaContainerWrapper::default();
    container.content_type = get_content_type_from_headers(req.headers_mut());
    let size = container.media_container.children().len();
    container.media_container.size = Some(size.try_into().unwrap());
    container.media_container.offset = original_offset;
    TransformBuilder::new(plex_client, params.clone())
        .with_transform(LibraryMixTransform {
            collection_ids,
            offset,
            limit,
        })
        .apply_to(&mut container)
        .await;
    res.render(container); // TODO: FIx XML
}
