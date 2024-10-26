#[macro_use]
extern crate tracing;

use replex::config::Config;
use replex::routes::*;
use salvo::prelude::*;
use std::env;
//use tonic::metadata::MetadataMap;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    let config: Config = Config::figment().extract().unwrap();

    // set default log level
    if let Err(i) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info")
    }

    let fmt_layer = tracing_subscriber::fmt::layer().map_fmt_fields(|f| f.debug_alt());
    let console_layer = match config.enable_console {
        true => Some(console_subscriber::spawn()),
        false => None,
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(console_layer)
        // .with(otlp_layer)
        .with(fmt_layer)
        .init();
        
    if config.host.is_none() {
        tracing::error!("REPLEX_HOST is required. Exiting");
        return;
    }
    if config.token.is_none() {
        tracing::warn!("REPLEX_TOKEN not defined. Hero art might not load correctly.");
    }

    tracing::debug!("Running with config: {:#?}", &config);
  
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
        let acceptor =
            TcpListener::new(format!("0.0.0.0:{}", config.port.unwrap_or(443)))
                .acme()
                .cache_path("/data/acme/letsencrypt")
                .add_domain(config.ssl_domain.unwrap())
                .bind()
                .await;
        Server::new(acceptor)
            //.idle_timeout(Duration::from_secs(60 * 101))
            .serve(router)
            .await;
    } else {
        let acceptor =
            TcpListener::new(format!("0.0.0.0:{}", config.port.unwrap_or(80)))
                .bind()
                .await;
        Server::new(acceptor)
            //.idle_timeout(Duration::from_secs(60 * 101))
            .serve(router)
            .await;
    }
}
