use crate::config::Config;
use crate::logging::*;
use crate::models::*;
use crate::plex_client::*;
use crate::timeout::*;
use crate::transform::*;
use crate::url::*;
use crate::utils::*;
use crate::webhooks;
use itertools::Itertools;
use salvo::compression::Compression;
use salvo::cors::Cors;
use salvo::http::header::CONTENT_TYPE;
use salvo::http::{Request, Response, StatusCode};
use salvo::prelude::*;
use salvo::routing::PathFilter;
use salvo::http::HeaderValue;
use salvo::http::header;
use tokio::time::Duration;
use url::Url;
use http;

pub fn route() -> Router {
    let config: Config = Config::figment().extract().unwrap();

    // cant use colon in paths. So we do it with an regex
    let guid = regex::Regex::new(":").unwrap();
    PathFilter::register_wisp_regex("colon", guid);

    let mut router = Router::with_hoop(Cors::permissive().into_handler())
        .hoop(Logger::new())
        .hoop(should_skip)
        .hoop(Timeout::new(Duration::from_secs(60 * 200)))
        .hoop(Compression::new().enable_gzip(CompressionLevel::Fastest));
    // .hoop(affix::insert("script_engine", Arc::new(script_engine)));

    if config.redirect_streams {
        router = router
            .push(
                Router::with_path(
                    "/video/<colon:colon>/transcode/universal/session/<**rest>",
                )
                .goal(redirect_stream),
            )
            .push(
                Router::with_path(
                    "/library/parts/<itemid>/<partid>/file.<extension>",
                )
                .goal(redirect_stream),
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
                    .goal(proxy_request),
            )
            .push(
                Router::with_path("/playQueues")
                    .hoop(disable_related_query)
                    .goal(proxy_request),
            );
    }

    let mut decision_router = Router::new()
        .path("/video/<colon:colon>/transcode/universal/decision")
        .goal(proxy_request);

    let mut start_router = Router::new()
        .path("/video/<colon:colon>/transcode/universal/start<**rest>")
        .goal(proxy_request);

    let mut subtitles_router = Router::new()
        .path("/video/<colon:colon>/transcode/universal/subtitles")
        .goal(proxy_request);

    // should go before force_maximum_quality and video_transcode_fallback
    if config.auto_select_version {
        decision_router = decision_router.hoop(auto_select_version);
        start_router = start_router.hoop(auto_select_version);
        subtitles_router = subtitles_router.hoop(auto_select_version);
    }

    if config.force_maximum_quality || config.disable_transcode {
        decision_router = decision_router.hoop(force_maximum_quality);
        start_router = start_router.hoop(force_maximum_quality);
        subtitles_router = subtitles_router.hoop(force_maximum_quality);
    }

    if config.video_transcode_fallback_for.is_some() {
        decision_router = decision_router.hoop(video_transcode_fallback);
        //subtitles_router = subtitles_router.hoop(video_transcode_fallback);
    }

    decision_router = decision_router.hoop(direct_stream_fallback);

    router = router
        .push(decision_router)
        .push(start_router)
        .push(subtitles_router);

    if config.disable_continue_watching {
        router = router.push(
            Router::new()
                .path(PLEX_CONTINUE_WATCHING)
                .get(empty_handler),
        );
    }
    
    if config.ntf_watchlist_force {
        router = router.push(
            Router::new()
                .hoop(ntf_watchlist_force)
                //.get(ping)
                //.hoop(debug)
                .goal(proxy_request)
                .path("/media/providers"),
        );
    }

    router = router
        .push(
            Router::new()
                .path(PLEX_HUBS_PROMOTED)
                .hoop(transform_req_content_directory)
                .hoop(transform_req_include_guids)
                .hoop(transform_req_android)
                .hoop(proxy_for_transform)
                .get(transform_hubs_response),
        )
        .push(
            Router::new()
                .path("/replex/test_proxy/<**rest>")
                .goal(test_proxy_request),
        )
        .push(
            Router::new()
                .path("/replex/image/hero/<type>/<uuid>")
                .get(hero_image)
        )
        .push(
            Router::new()
                .path(format!("{}/<id>", PLEX_HUBS_SECTIONS))
                .hoop(transform_req_include_guids)
                .hoop(transform_req_android)
                .hoop(proxy_for_transform)
                .get(transform_hubs_response)
        )
        .push(
            Router::new()
                .path("/replex/webhooks")
                .post(webhook_plex),
        )
        .push(
            Router::new()
                .path("/ping")
                .get(ping),
        )
        .push(
            Router::new()
                .path("/replex/<style>/library/collections/<ids>/children")
                .get(get_collections_children),
        )
        .push(
            Router::new()
                .path("/replex/<style>/<**rest>")
                .get(default_transform),
        )
        .push(
            Router::with_path("/photo/<colon:colon>/transcode")
                .hoop(fix_photo_transcode_request)
                .hoop(resolve_local_media_path)
                .goal(proxy_request),
        )
        .push(Router::with_path("<**rest>").goal(proxy_request));

    router
}

#[handler]
async fn proxy_request(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
    ctrl: &mut FlowCtrl,
) {
    let proxy = default_proxy();
    proxy.handle(req, depot, res, ctrl).await;
}

#[handler]
async fn test_proxy_request(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
    ctrl: &mut FlowCtrl,
) {
    let proxy = test_proxy("https://webhook.site".to_string());
    proxy.handle(req, depot, res, ctrl).await;
}

#[handler]
async fn proxy_for_transform(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
    ctrl: &mut FlowCtrl,
) -> Result<(), anyhow::Error> {
    let proxy = default_proxy();
    let headers_ori = req.headers().clone();
    req.headers_mut().insert(http::header::ACCEPT, header::HeaderValue::from_static("application/json"));
    proxy.handle(req, depot, res, ctrl).await;
    *req.headers_mut() = headers_ori;
    Ok(())
}

// skip processing when product is plexamp
#[handler]
async fn should_skip(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
    ctrl: &mut FlowCtrl,
) {
    let context: PlexContext = req.extract().await.unwrap();
    
    let is_livetv = match context.path.clone() {
        Some(v) => v.contains("livetv"),
        None => false
    };
    
    let is_plexamp = match context.product.clone() {
        Some(v) => v.to_lowercase().contains("plexamp"),
        None => false
    };

    if is_livetv || is_plexamp {
        let config: Config = Config::dynamic(req).extract().unwrap();
        let proxy = default_proxy();

        proxy.handle(req, depot, res, ctrl).await;
        ctrl.skip_rest();
    }
}

#[handler]
async fn redirect_stream(
    req: &mut Request,
    _depot: &mut Depot,
    res: &mut Response,
) {
    let config: Config = Config::dynamic(req).extract().unwrap();
    let redirect_url = if config.redirect_streams_host.clone().is_some() {
        format!(
            "{}{}",
            config.redirect_streams_host.clone().unwrap(),
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
    let context: PlexContext = req.extract().await.unwrap();
    if context.size.is_some() && context.clone().size.unwrap().contains('-')
    // (catched things like (medlium-240, large-500),i dont think size paramater orks at all, but who knows
    // && context.platform.is_some()
    // && context.clone().platform.unwrap().to_lowercase() == "android"
    {
        let size: String = context
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
        //add_query_param_salvo(req, "quality".to_string(), "80".to_string());
    }
}

// resolve a local media path to full url
#[handler]
async fn resolve_local_media_path(
    req: &mut Request,
    res: &mut Response,
) {
    let mut context: PlexContext = req.extract().await.unwrap();
    let url = req.query::<String>("url");
    if url.is_some() && url.clone().unwrap().contains("/replex/image/hero")
    {
        let uri: url::Url = url::Url::parse(url.unwrap().as_str()).unwrap();
        let segments = uri.path_segments().unwrap().collect::<Vec<&str>>();
        
        let uuid = segments.last().unwrap().replace(".jpg", "");
        //if context.token.is_none() {
        //    context.token = Some(segments.last().unwrap().to_string());
        //}

        let plex_client = PlexClient::from_context(&context);
        let rurl = plex_client.get_hero_art(uuid.to_string()).await;
        if rurl.is_some() {
          add_query_param_salvo(req, "url".to_string(), rurl.unwrap());
        }
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
async fn debug(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    //dbg!("tequested");
    let context: PlexContext = req.extract().await.unwrap();
    dbg!(&context.token);
    //dbg!(&req);
}

#[handler]
async fn ntf_watchlist_force(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    // use memory_stats::memory_stats;
    // dbg!(memory_stats().unwrap().physical_mem / 1024 / 1000);
    let context: PlexContext = req.extract().await.unwrap();
    if context.clone().token.is_some() {
        tokio::spawn(async move {
            let token = context.clone().token.unwrap();
            let client_id = context.clone().client_identifier.unwrap();
            let url = format!("https://notifications.plex.tv/api/v1/notifications/settings?X-Plex-Token={}", &token);
            let json_data = r#"{"enabled": true,"libraries": [],"identifier": "tv.plex.notification.library.new"}"#;
            let client = reqwest::Client::new();
        
            tracing::info!(
                username = %context.clone().username.unwrap_or_default(),
                platform = %context.clone().product.unwrap_or_default(),
                platform = %context.clone().device_name.unwrap_or_default(),
                "Bootstrao for request"
            );
        
            let client_base = "https://clients.plex.tv";
            let res = client
                    .get(format!("{}/api/v2/user", client_base))
                    .header("Accept", "application/json")
                    .header("X-Plex-Token", &token)
                    .header("X-Plex-Client-Identifier", &client_id)
                    .send()
                    .await
                    .unwrap();
                    
        
            if !res.status().is_success() {
              tracing::info!(
                "cannot get user"
              );
              return;
            }
            
            
            let user: PlexUser = res.json().await.unwrap();
            tracing::info!(
                id = %user.id,
                uuid = %user.uuid,
                username = %user.username,
                "got user"
            );
            
            let response = client
                .post(url)
                .header("Content-Type", "application/json")
                .body(json_data.to_owned())
                .send()
                .await
                .unwrap();
        
            tracing::info!(
                status = %response.status(),
                "watchlist status"
            );
            
            let opts = vec![
              "tv.plex.provider.vod",
              "tv.plex.provider.music",
            ];
            
            //let 
            //return;
            let u = format!("{}/api/v2/user/{}/settings/opt_outs", client_base, &user.uuid);
            for key in opts {
                let response = client
                    .post(format!("{}?key={}&value=opt_out", u.clone(), key))
                    .header("Accept", "application/json")
                    .header("X-Plex-Token", &token)
                    .header("X-Plex-Client-Identifier", &client_id)
                    .send()
                    .await
                    .unwrap();
  
                tracing::info!(
                status = %response.status(),
                "opt out status"
                );
              
            }
            
        });
    }
}

#[handler]
pub async fn empty_handler(
    req: &mut Request,
    res: &mut Response,
) -> Result<(), anyhow::Error> {
    let content_type = get_content_type_from_headers(req.headers_mut());
    let mut container: MediaContainerWrapper<MediaContainer> =
        MediaContainerWrapper::default();
    container.content_type = content_type.clone();
    // container.media_container.size = Some(0);
    container.media_container.identifier =
        Some("com.plexapp.plugins.library".to_string());
    res.render(container);
    return Ok(());
}

#[handler]
pub async fn webhook_plex(
    req: &mut Request,
    res: &mut Response,
) -> Result<(), anyhow::Error> {
    dbg!("YOOO");
    let raw = req.form::<String>("payload").await;
    let payload: webhooks::Payload = serde_json::from_str(&raw.unwrap())?;
    dbg!(&req);
    dbg!(payload);

    // watchlist();
    res.render(());
    return Ok(());
}

#[handler]
pub async fn hero_image(
    req: &mut Request,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
    depot: &mut Depot,
) {
    let context: PlexContext = req.extract().await.unwrap();
    let t = req.param::<String>("type").unwrap();
    let uuid = req.param::<String>("uuid").unwrap();

    let plex_client = PlexClient::from_context(&context);
    let url = plex_client.get_hero_art(uuid).await;
    if url.is_none() {
        res.status_code(StatusCode::NOT_FOUND);
        return
    }
    // let uri = url.unwrap().parse::<http::Uri>().unwrap();;
    // req.set_uri(uri);
    // let proxy = proxy("https://metadata-static.plex.tv".to_string());
    // proxy.handle(req, depot, res, ctrl).await;

    res.render(Redirect::found(url.unwrap()));
}

// if directplay fails we remove it.
#[handler]
pub async fn direct_stream_fallback(
    req: &mut Request,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
    depot: &mut Depot,
) -> Result<(), anyhow::Error> {
    let config: Config = Config::dynamic(req).extract().unwrap();
    let context: PlexContext = req.extract().await.unwrap();
    let plex_client = PlexClient::from_context(&context);
    let queries = req.queries().clone();

    let direct_play = queries
        .get("directPlay")
        .unwrap_or(&"1".to_string())
        .to_owned();

    if direct_play != "1" {
        return Ok(());
    }
    
    let mut res_upstream = &mut Response::new();
    proxy_for_transform.handle(req, depot, res_upstream, ctrl).await;

    match res_upstream.status_code.unwrap() {
        http::StatusCode::OK => {
            let container: MediaContainerWrapper<MediaContainer> =
            //from_reqwest_response(upstream_res).await?;
            from_salvo_response(res_upstream).await?;
    
            if container.media_container.general_decision_code.is_some()
                && container.media_container.general_decision_code.unwrap() == 2000
            {
                tracing::debug!(
                    "Direct play not avaiable, falling back to direct stream"
                );
                add_query_param_salvo(req, "directPlay".to_string(), "0".to_string());
                add_query_param_salvo(req, "directStream".to_string(), "1".to_string());
            };
            //return Ok(());
        },
        http::StatusCode::BAD_REQUEST => {
            tracing::debug!(
                "Got 400 bad request, falling back to direct stream"
            );
            add_query_param_salvo(req, "directPlay".to_string(), "0".to_string());
            add_query_param_salvo(req, "directStream".to_string(), "1".to_string());   
            //return Ok(());   
        },
        status => {
            tracing::error!(status = ?status, res = ?res_upstream, "Failed to get plex response");
            return Err(
                salvo::http::StatusError::internal_server_error().into()
            );
        }
    };
    //res = &mut Response::new();
    return Ok(());
}

#[handler]
pub async fn transform_hubs_response(
    req: &mut Request,
    res: &mut Response,
) -> Result<(), anyhow::Error> {
    let context: PlexContext = req.extract().await.unwrap();
    let plex_client = PlexClient::from_context(&context);
    let content_type = get_content_type_from_headers(req.headers_mut());

    let mut container: MediaContainerWrapper<MediaContainer> =
        from_salvo_response(res).await?;
    container.content_type = content_type;

    TransformBuilder::new(plex_client, context.clone())
        .with_transform(HubRestrictionTransform)
        .with_transform(HubStyleTransform { is_home: true })
        .with_transform(HubWatchedTransform)
        .with_transform(HubInterleaveTransform)
        .with_transform(UserStateTransform)
        .with_transform(HubKeyTransform)
        .apply_to(&mut container)
        .await;

    res.render(container);
    Ok(())
}

#[handler]
pub async fn transform_req_content_directory(
    req: &mut Request,
    res: &mut Response,
    ctrl: &mut FlowCtrl
) {
    let config: Config = Config::dynamic(req).extract().unwrap();
    let context: PlexContext = req.extract().await.unwrap();
    let plex_client = PlexClient::from_context(&context);
    let content_type = get_content_type_from_headers(req.headers_mut());

    if context.clone().pinned_content_directory_id.is_some()
        && context.clone().content_directory_id.unwrap()[0]
            != context.clone().pinned_content_directory_id.unwrap()[0]
    {
        // We only fill the first one.
        let mut container: MediaContainerWrapper<MediaContainer> =
            MediaContainerWrapper::default();
        container.content_type = content_type.clone();
        container.media_container.size = Some(0);
        // container.media_container.allow_sync = Some("1".to_string());
        container.media_container.identifier =
            Some("com.plexapp.plugins.library".to_string());
        res.render(container);
        ctrl.skip_rest();
        return;
    }

    if context.clone().pinned_content_directory_id.is_some() {
        // first directory, load everything here because we wanna reemiiiixxx
        add_query_param_salvo(
            req,
            "contentDirectoryID".to_string(),
            context
                .clone()
                .pinned_content_directory_id
                .clone()
                .unwrap()
                .iter()
                .join(",")
                .to_string(),
        );
    }
}

#[handler]
pub async fn transform_req_include_guids(
    req: &mut Request,
    res: &mut Response,
) {
    add_query_param_salvo(req, "includeGuids".to_string(), "1".to_string());
}

// some androids have trouble loading more for hero style. So load more at once
#[handler]
pub async fn transform_req_android(
    req: &mut Request,
    res: &mut Response,
) {
    let config: Config = Config::dynamic(req).extract().unwrap();
    let context: PlexContext = req.extract().await.unwrap();
    
    let mut count = context.clone().count.unwrap_or(25);
    match context.platform.unwrap() {
        Platform::Android => count = 50,
        _ => (),
    }
    // Hack, as the list could be smaller when removing watched items. So we request more.
    if config.exclude_watched && count < 50 {
        count = 50;
    }

    add_query_param_salvo(req, "count".to_string(), count.to_string());
}


// rhis handles refresh of individual rows or paging and paging if it
#[handler]
pub async fn get_collections_children(
    req: &mut Request,
    _depot: &mut Depot,
    res: &mut Response,
) -> Result<(), anyhow::Error> {
    let config: Config = Config::dynamic(req).extract().unwrap();
    let context: PlexContext = req.extract().await.unwrap();
    let collection_ids = req.param::<String>("ids").unwrap();
    let collection_ids: Vec<u32> = collection_ids
        .split(',')
        .filter(|&v| !v.parse::<u32>().is_err())
        .map(|v| v.parse().unwrap())
        .collect();
    let plex_client = PlexClient::from_context(&context);
    let content_type = get_content_type_from_headers(req.headers_mut());

    // We dont listen to pagination. We have a hard max of 250 per collection
    let mut limit: i32 = 250;
    let mut offset: i32 = 0;

    // in we dont remove watched then we dont need to limit
    if !config.exclude_watched {
        limit = context.container_size.unwrap_or(50);
        offset = context.container_start.unwrap_or(0);
    }

    // create a stub
    let mut container: MediaContainerWrapper<MediaContainer> =
        MediaContainerWrapper::default();
    container.content_type = content_type;
    let size = container.media_container.children().len();
    container.media_container.size = Some(size.try_into().unwrap());
    container.media_container.offset = Some(offset);

    // filtering of watched happens in the transform
    TransformBuilder::new(plex_client, context.clone())
        .with_transform(LibraryInterleaveTransform {
            collection_ids: collection_ids.clone(),
            offset,
            limit,
        })
        .with_transform(HubReorderTransform {
            collection_ids: collection_ids.clone()
       })
        .with_transform(HubRestrictionTransform)
        .with_transform(CollectionStyleTransform {
            collection_ids: collection_ids.clone(),
            hub: context.content_directory_id.is_some() // its a guessing game
                && !context.include_collections
                && !context.include_advanced
                && !context.exclude_all_leaves,
        })
        .with_transform(UserStateTransform)
        //.with_transform(MediaContainerScriptingTransform)
        .apply_to(&mut container)
        .await;

    res.render(container); // TODO: FIx XML
    Ok(())
}

#[handler]
pub async fn default_transform(
    req: &mut Request,
    _depot: &mut Depot,
    res: &mut Response,
) -> Result<(), anyhow::Error> {
    let config: Config = Config::dynamic(req).extract().unwrap();
    let context: PlexContext = req.extract().await.unwrap();
    let plex_client = PlexClient::from_context(&context);
    let content_type = get_content_type_from_headers(req.headers_mut());
    let style = req.param::<Style>("style").unwrap();
    let rest_path = req.param::<String>("**rest").unwrap();

    // We dont listen to pagination. We have a hard max of 250 per collection
    let mut limit: i32 = 250;
    let mut offset: i32 = 0;

    // in we dont remove watched then we dont need to limit
    if !config.exclude_watched {
        limit = context.container_size.unwrap_or(50);
        offset = context.container_start.unwrap_or(0);
    }

    let mut url = Url::parse(req.uri_mut().to_string().as_str()).unwrap();
    url.set_path(&rest_path);
    req.set_uri(hyper::Uri::try_from(url.as_str()).unwrap());
    
    
    // patch, plex seems to pass wrong contentdirid, probaply cause we all load it inti the first
    let mut queries = req.queries().clone();
    queries.remove("contentDirectoryID");
    replace_query(queries, req);

    let upstream_res = plex_client.request(req).await?;
    match upstream_res.status() {
        reqwest::StatusCode::OK => (),
        status => {
            tracing::error!(status = ?status, res = ?upstream_res, req = ?req, "Failed to get plex response");
            return Err(
                salvo::http::StatusError::internal_server_error().into()
            );
        }
    };

    let mut container: MediaContainerWrapper<MediaContainer> =
        from_reqwest_response(upstream_res).await?;
    container.content_type = content_type;

    TransformBuilder::new(plex_client, context.clone())
        .with_transform(HubRestrictionTransform)
        .with_transform(MediaStyleTransform { style: style })
        .with_transform(UserStateTransform)
        .with_transform(HubWatchedTransform)
        .with_transform(HubKeyTransform)
        .apply_to(&mut container)
        .await;

    res.render(container);
    Ok(())
}

#[handler]
pub async fn get_library_item_metadata(req: &mut Request, res: &mut Response) {
    let config: Config = Config::dynamic(req).extract().unwrap();
    let context: PlexContext = req.extract().await.unwrap();
    let plex_client = PlexClient::from_context(&context);
    let content_type = get_content_type_from_headers(req.headers_mut());

    if config.disable_related {
        add_query_param_salvo(
            req,
            "includeRelated".to_string(),
            "0".to_string(),
        );
    }

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

    TransformBuilder::new(plex_client, context.clone())
        //.with_transform(MediaContainerScriptingTransform)
        .apply_to(&mut container)
        .await;
    // dbg!(container.media_container.count);
    res.render(container);
}

// const RESOLUTIONS: HashMap<&'static str, &'static str> =
//     HashMap::from([("1080p", "1920x1080"), ("4k", "4096x2160")]);

#[handler]
async fn force_maximum_quality(req: &mut Request) -> Result<(), anyhow::Error> {
    let context: PlexContext = req.extract().await.unwrap();
    let plex_client = PlexClient::from_context(&context);
    let config: Config = Config::dynamic(req).extract().unwrap();
    let mut queries = req.queries().clone();

    if queries.get("maxVideoBitrate").is_none() && queries.get("videoBitrate").is_none() {
        return Ok(())
    }

    queries.remove("maxVideoBitrate");
    queries.remove("videoBitrate");
    queries.remove("autoAdjustQuality");
    queries.insert("autoAdjustQuality".to_string(), "0".to_string());
    queries.remove("directStream");
    queries.insert("directStream".to_string(), "1".to_string());
    queries.remove("directPlay");
    queries.insert("directPlay".to_string(), "1".to_string());
    queries.remove("videoQuality");
    queries.insert("videoQuality".to_string(), "100".to_string());
    // queries.insert("directStreamAudio".to_string(), "0".to_string());
    //queries.remove("videoResolution");
    //queries.insert("videoResolution".to_string(), "4096x2160".to_string());

    // some clients send wrong buffer format
    if let Some(size) = queries.remove("mediaBufferSize") {
        queries.insert(
            "mediaBufferSize".to_string(),
            (size[0].parse::<f32>().unwrap() as i64).to_string(),
        );
    }
    // if let Some(i) = req.queries().get("protocol") {
    //     if i == "http" {
    //         queries.remove("copyts");
    //         queries.insert("copyts".to_string(), "0".to_string());
    //         queries.remove("hasMDE");
    //         queries.insert("hasMDE".to_string(), "0".to_string());
    //     }
    // }

    let query_key = "X-Plex-Client-Profile-Extra".to_string();
    if queries.contains_key(&query_key) {
        let extra = &queries.remove(&query_key.clone()).unwrap()[0];

        let filtered_extra = extra
            .split("+")
            .filter(|s| {
                !s.contains("add-limitation")
                    && !s.to_lowercase().contains("name=video.bitrate")
            })
            .join("+");

        queries.insert(query_key, filtered_extra);
    };

    if config.force_direct_play_for.is_some() && queries.get("path").is_some() {
        let resos = config.force_direct_play_for.unwrap();
        let item = plex_client
            .clone()
            .get_item_by_key(req.queries().get("path").unwrap().to_string())
            .await
            .unwrap();

        let media_index: usize = if req.queries().get("mediaIndex").is_none()
            || req.queries().get("mediaIndex").unwrap() == "-1"
        {
            0
        } else {
            req.queries()
                .get("mediaIndex")
                .unwrap()
                .parse::<usize>()
                .unwrap()
        };

        let media_item =
            item.media_container.metadata[0].media[media_index].clone();

        for reso in resos {
            if let Some(video_resolution) = media_item.video_resolution.clone()
            {
                if video_resolution.to_lowercase() == reso.to_lowercase() {
                    queries.remove("directPlay");
                    queries.insert("directPlay".to_string(), "1".to_string());
                    queries.remove("videoResolution");
                    // queries.insert(
                    //     "videoResolution".to_string(),
                    //     RESOLUTIONS.get(&reso.to_lowercase()),
                    // );
                }
            }
        }
    }

    replace_query(queries, req);
    Ok(())
}

// async fn execute_video_transcode_fallback(
//     req: &mut Request,
//     item: MediaContainerWrapper<MediaContainer>,
//     media_index: usize,
// ) -> Result<(), anyhow::Error> {
//     let context: PlexContext = req.extract().await.unwrap();
//     let plex_client = PlexClient::from_context(&context);
//     let mut queries = req.queries().clone();
//     let mut original_queries = req.queries().clone();

//     let response = plex_client.request(req).await?;
//     let mut transcode: MediaContainerWrapper<MediaContainer> =
//         from_reqwest_response(response).await?;

//     let streams =
//         &transcode.media_container.metadata[0].media[media_index].parts[0].streams;
//     let selected_media = transcode.media_container.metadata[0].media[media_index].clone();
//     let mut fallback_selected = false;
//     for stream in streams {
//         if stream.stream_type.clone().unwrap() == 1
//             && stream.decision.clone().unwrap_or("unknown".to_string())
//                 == "transcode"
//         {
//             tracing::trace!(
//                 "{} is transcoding, looking for fallback",
//                 selected_media
//             );
//             // for now just select a random fallback
//             for (index, media) in
//                 item.media_container.metadata[0].media.iter().enumerate()
//             {
//                 if transcode.media_container.metadata[0].media[media_index].id != media.id
//                 {
//                     tracing::debug!(
//                         "Video transcode fallback from {} to {}",
//                         selected_media,
//                         media,
//                     );
//                     queries.remove("mediaIndex");
//                     queries.insert("mediaIndex".to_string(), index.to_string());
//                     queries.remove("directPlay");
//                     queries.insert("directPlay".to_string(), "0".to_string());
//                     queries.remove("directStream");
//                     queries.insert("directStream".to_string(), "1".to_string());
//                     fallback_selected = true;
//                     break;
//                 }
//             }
//         }
//     }
//     if !fallback_selected {
//         replace_query(original_queries, req);
//     }
//     Ok(())
// }

pub struct TranscodingStatus {
    pub is_transcoding: bool,
    pub decision_result: MediaContainerWrapper<MediaContainer>,
}

async fn get_transcoding_for_request(
    req: &mut Request,
) -> Result<TranscodingStatus, anyhow::Error> {
    let context: PlexContext = req.extract().await.unwrap();
    let plex_client = PlexClient::from_context(&context);
    
    let mut res = &mut Response::new();
    let mut depot = &mut Depot::new();
    let mut ctrl = &mut FlowCtrl::new(vec![]);
    proxy_for_transform.handle(req, depot, res, ctrl).await;
    dbg!(&res);
    
    let ress = plex_client.proxy_request(&req).await?;
    dbg!(&ress);
    //dbg!(&req);

    let transcode: MediaContainerWrapper<MediaContainer> =
        from_salvo_response(res).await?;
    let mut is_transcoding = false;

    if transcode.media_container.size.is_some()
        && transcode.media_container.size.unwrap() == 0
    {
        return Ok(TranscodingStatus {
            is_transcoding,
            decision_result: transcode,
        });
    }

    let streams =
        &transcode.media_container.metadata[0].media[0].parts[0].streams;
    // let selected_media = transcode.media_container.metadata[0].media[0].clone();
    for stream in streams {
        if stream.stream_type.clone().unwrap() == 1
            && stream.decision.clone().unwrap_or("unknown".to_string())
                == "transcode"
        {
            is_transcoding = true;
            break;
        }
    }

    Ok(TranscodingStatus {
        is_transcoding,
        decision_result: transcode,
    })
}

// TODO: Fallback to a version close to the requested bitrate
#[handler]
async fn video_transcode_fallback(
    req: &mut salvo::Request,
    depot: &mut Depot,
    res: &mut salvo::Response,
    ctrl: &mut FlowCtrl,
) -> Result<(), anyhow::Error> {
    let context: PlexContext = req.extract().await.unwrap();
    let plex_client = PlexClient::from_context(&context);
    let config: Config = Config::dynamic(req).extract().unwrap();
    let mut queries = req.queries().clone();
    let original_queries = req.queries().clone();
    let media_index: usize = if req.queries().get("mediaIndex").is_none()
        || req.queries().get("mediaIndex").unwrap() == "-1"
    {
        0
    } else {
        req.queries()
            .get("mediaIndex")
            .unwrap()
            .parse::<usize>()
            .unwrap()
    };

    let fallback_for =
        config.video_transcode_fallback_for.unwrap()[0].to_lowercase();

    let item = plex_client
        .clone()
        .get_item_by_key(req.queries().get("path").unwrap().to_string())
        .await
        .unwrap();

    if item.media_container.metadata[0].media[media_index]
        .video_resolution
        .clone()
        .unwrap()
        .to_lowercase()
        != fallback_for
    {
        tracing::debug!("Media item not marked for fallback, continue playing");
        return Ok(());
    }

    if item.media_container.metadata[0].media.len() <= 1 {
        tracing::debug!("Nothing to fallback on, skipping fallback check");
    } else {
        // execute_video_transcode_fallback(req, item, media_index).await?;
        // let response = plex_client.request(req).await?;
        // let mut transcode: MediaContainerWrapper<MediaContainer> =
        //     from_reqwest_response(response).await?;
        // let streams =
        //     &transcode.media_container.metadata[0].media[0].parts[0].streams;
        // let selected_media =
        //     transcode.media_container.metadata[0].media[0].clone();
        let mut requested_bitrate: Option<i64> = None;
        if queries.get("videoBitrate").is_some() {
            requested_bitrate =
                Some(queries.get("videoBitrate").unwrap().parse().unwrap());
        } else if queries.get("maxVideoBitrate").is_some() {
            requested_bitrate =
                Some(queries.get("maxVideoBitrate").unwrap().parse().unwrap());
        }

        let mut fallback_selected = false;
        // this could fail.
        let status: TranscodingStatus =
            get_transcoding_for_request(req).await?;
        let selected_media = item.media_container.metadata[0].media[0].clone();
        let mut available_media_ids: Vec<i64> = item.media_container.metadata
            [0]
        .media
        .iter()
        .map(|x| x.id)
        .collect();
        available_media_ids.retain(|x| *x != selected_media.id);
        // available_media_ids.remove(selected_media.id);
        if status.is_transcoding {
            tracing::debug!(
                "{} transcoding, looking for fallback",
                selected_media
            );

            let mut media_items =
                item.media_container.metadata[0].media.clone();
            media_items.sort_by(|x, y| {
                let current_density = x.height.unwrap() * x.width.unwrap();
                let next_density = y.height.unwrap() * y.width.unwrap();

                if current_density < next_density {
                    return std::cmp::Ordering::Greater;
                } else {
                    return std::cmp::Ordering::Less;
                }
            });
            // dbg!(&media_items.iter().map(|x| x.video_resolution.clone()));
            // for now just select a random fallback
            for (index, media) in media_items.iter().enumerate() {
                if available_media_ids.contains(&media.id) {
                    if queries.get("maxVideoBitrate").is_some()
                        || queries.get("videoBitrate").is_some()
                    {
                        // tracing::trace!(
                        //     "Video has max bitrate which always forces transcode. Forcing max quality for fallback {}",
                        //     media,
                        // );

                        // if same resolution we can assume it will transcode again. Fallback to another resolution
                        let resolution = media
                            .video_resolution
                            .clone()
                            .unwrap()
                            .to_lowercase();
                        if resolution == fallback_for {
                            continue;
                        }

                        // check if requested falls into a resolution range. Either we remove the max bitrate or allow it
                        //let requested_bitrate: i64 = queries
                        //    .get("videoBitrate")
                        //    .unwrap_or_else(|| queries.get("maxVideoBitrate").unwrap()).parse().unwrap();

                        //if (resolution == "1080" && requested_bitrate >= 8000)
                        //    || (resolution == "720"
                        //        && requested_bitrate >= 2000)
                        //{
                        //    force_maximum_quality
                        //        .handle(req, depot, res, ctrl)
                        //        .await;
                        //    queries = req.queries().clone();
                        //}
                    }

                    // force_maximum_quality
                    tracing::debug!(
                        "Video transcode fallback from {} to {}",
                        selected_media,
                        media,
                    );
                    // let mut media_queries = req.queries().clone();
                    queries.remove("mediaIndex");
                    queries.insert("mediaIndex".to_string(), index.to_string());
                    queries.remove("directStream");
                    queries.insert("directStream".to_string(), "1".to_string());

                    if requested_bitrate.is_none() {
                        queries.remove("directPlay");
                        queries
                            .insert("directPlay".to_string(), "1".to_string());
                    }

                    queries.remove("subtitles");
                    queries.insert("subtitles".to_string(), "auto".to_string());

                    replace_query(queries.clone(), req);
                    // processed_media_indexes.append(selected_media.id);
                    // available_media_ids.remove(selected_media.id);

                    if media.video_resolution.clone().unwrap().to_lowercase()
                        != fallback_for
                    {
                        fallback_selected = true;
                        break;
                    }

                    let status: TranscodingStatus =
                        get_transcoding_for_request(req).await?;
                    available_media_ids.retain(|x| *x != media.id);
                    if status.is_transcoding && available_media_ids.len() != 0 {
                        tracing::debug!(
                            "Fallback is transcoding, getting another fallback",
                        );
                        continue;
                    }
                    // let mut transcode: MediaContainerWrapper<MediaContainer> =
                    //     from_reqwest_response(response).await?;
                    fallback_selected = true;
                    break;
                }
            }
            if !fallback_selected {
                tracing::debug!("No suitable fallback found");
                replace_query(original_queries, req);
            }
        }
    }

    // replace_query(queries, req);
    Ok(())
}

/// When multiple qualities are avaiable, select the most relevant one.
/// Does not work for every client as some client decides themselfs which version to use.
#[handler]
async fn auto_select_version(req: &mut Request) {
    let context: PlexContext = req.extract().await.unwrap();
    let plex_client = PlexClient::from_context(&context);
    let mut queries = req.queries().clone();
    let media_index = queries.get("mediaIndex");

    if media_index.is_some() && media_index.unwrap() != "-1" {
        tracing::debug!(
            "Skipping auto selected as client specified a media index"
        );
        return;
    }

    if context.screen_resolution.len() == 0 {
        tracing::debug!(
            "Skipping auto selected as no screen resolution has been specified"
        );
        return;
    }

    if queries.get("path").is_some() {
        let item = plex_client
            .get_item_by_key(req.queries().get("path").unwrap().to_string())
            .await
            .unwrap();

        if item.media_container.metadata[0].media.len() <= 1 {
            tracing::debug!(
                "Only one media version available, skipping auto select"
            );
            return;
        }

        let mut requested_bitrate: Option<i64> = None;
        if queries.get("videoBitrate").is_some() {
            requested_bitrate =
                Some(queries.get("videoBitrate").unwrap().parse().unwrap());
        } else if queries.get("maxVideoBitrate").is_some() {
            requested_bitrate =
                Some(queries.get("maxVideoBitrate").unwrap().parse().unwrap());
        }

        let mut media = item.media_container.metadata[0].media.clone();
        let device_density = context.screen_resolution[0].height
            * context.screen_resolution[0].width;
        if media.len() > 1 {
            media.sort_by(|x, y| {
                let current_density = x.height.unwrap() * x.width.unwrap();
                let next_density = y.height.unwrap() * y.width.unwrap();
                let q = current_density - device_density;
                let qq = next_density - device_density;

                if q > qq {
                    return std::cmp::Ordering::Greater;
                } else {
                    return std::cmp::Ordering::Less;
                }
            })
        }

        for (index, m) in
            item.media_container.metadata[0].media.iter().enumerate()
        {
            if m.id == media[0].id {
                tracing::debug!("Auto selected {}", m);
                queries.remove("mediaIndex");
                queries.insert("mediaIndex".to_string(), index.to_string());
                // directPlay is meant for the first media item
                if requested_bitrate.is_none() {
                    queries.remove("directPlay");
                    queries.insert("directPlay".to_string(), "1".to_string());
                }

                queries.remove("subtitles");
                queries.insert("subtitles".to_string(), "auto".to_string());

            }
        }
    }
    replace_query(queries, req);
}

#[handler]
async fn ping(req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    res.render("pong!")
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
