#[macro_use]
extern crate tracing;

use opentelemetry_otlp::WithExportConfig;
use replex::config::Config;
use replex::routes::*;
use salvo::prelude::*;
use std::env;
use std::time::Duration;
use tokio::{task, time};
use tonic::metadata::MetadataMap;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    let config: Config = Config::figment().extract().unwrap();
    if config.host.is_none() {
        tracing::error!("REPLEX_HOST is required. Exiting");
        return;
    }

    // set default log level
    if let Err(i) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info")
    }

    let fmt_layer = tracing_subscriber::fmt::layer();
    let console_layer = match config.enable_console {
        true => Some(console_subscriber::spawn()),
        false => None,
    };

    let otlp_layer = if config.newrelic_api_key.is_some() {
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
        Some(tracing_opentelemetry::layer().with_tracer(tracer))
    } else {
        None
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(console_layer)
        .with(otlp_layer)
        .with(fmt_layer)
        .init();

    // spawn our background task
    // let mut plex_client = PlexClient::dummy();
    // tokio::spawn(async move {
    //     let mut interval = time::interval(Duration::from_secs(60));
    //     loop {
    //         interval.tick().await;
    //         dbg!("we are being runned");
    //     }
    // });
    let version = env!("CARGO_PKG_VERSION");
    tracing::info!("Replex version {}", version);
    // dbg!(&config);

    let router = route();
    if config.ssl_enable && config.ssl_domain.is_some() {
        let acceptor = TcpListener::new("0.0.0.0:443")
            .acme()
            .cache_path("/data/acme/letsencrypt")
            .add_domain(config.ssl_domain.unwrap())
            .bind()
            .await;
        Server::new(acceptor)
            .idle_timeout(Duration::from_secs(60))
            .serve(router)
            .await;
    } else {
        let acceptor = TcpListener::new(format!("0.0.0.0:{}", config.port))
            .bind()
            .await;
        Server::new(acceptor)
            .idle_timeout(Duration::from_secs(60))
            .serve(router)
            .await;
    }
}
