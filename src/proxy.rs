use hyper::HeaderMap;
// use hyper::header::CONNECTION;
use salvo::http::ReqBody;
use salvo::http::ResBody;
use salvo::BoxedError;
use salvo::Error;
// use salvo::extract;
use salvo::http::header::HeaderValue;

// use reqwest::Client;
use salvo::http::header::CONNECTION;
use salvo::http::header::UPGRADE;
use salvo::prelude::*;
use salvo::proxy::Proxy as SalvoProxy;
use salvo::proxy::Upstreams;
use salvo::rt::TokioIo;
use tokio::io::copy_bidirectional;
use tracing::debug;

type HyperRequest = hyper::Request<ReqBody>;
type HyperResponse = hyper::Response<ResBody>;

pub struct PlexProxy<U> {
    pub proxy: SalvoProxy<U>,
}

impl<U> PlexProxy<U>
where
    U: Upstreams,
    U::Error: Into<BoxedError>,
{
    pub fn new(upstreams: U) -> Self {
        PlexProxy {
            proxy: SalvoProxy::new(upstreams),
        }
    }

    pub async fn request(&self, req: &mut Request) -> Response {
        let mut depot = Depot::with_capacity(1);
        let mut res = Response::new();
        let mut ctrl = FlowCtrl::new(vec![]);
        // disable gzip
        req.headers_mut()
            .insert("Accept-Encoding", HeaderValue::from_static("identity"));
        debug!("Making request: {:?}", &req);
        self.handle(req, &mut depot, &mut res, &mut ctrl).await;
        res
    }

    // // pub async fn websocket(&self, req: &mut Request, res: &mut Response) -> Result<HyperResponse, Error> {
    // pub async fn websocket(&self, req: &mut Request, res: &mut Response) {
    //     // let mut hyper_req = HyperRequest::new(req.take_body());
    //     // *hyper_req.extensions_mut() = req.extensions_mut();
    //     // *hyper_req.headers_mut() = req.headers_mut();
    //     // TryFrom::try_from(req.uri_mut()).map_err(Error::other)?;
    //     let mut request = hyper::Request::builder()
    //         .method(req.method())
    //         .uri(TryFrom::try_from(req.uri_mut()).map_err(Error::other)?);
    //     for (key, value) in req.headers() {
    //         request = request.header(key, value);
    //     }
    //     request.body(req.take_body()).map_err(Error::other);

    //     let client = Client::new();
    //     // let reqq: &mut HyperRequest = req.into();
    //     let mut response = client
    //         .execute(request.into())
    //         .await
    //         .map_err(Error::other)?;

    //     // let request_upgraded = req;
    //     // let response_upgraded = hyper::upgrade::on(res).await;
    //     // tokio::spawn(async move {
    //     //     match request_upgraded.await {
    //     //         Ok(request_upgraded) => {
    //     //             let mut request_upgraded = TokioIo::new(request_upgraded);
    //     //             if let Err(e) =
    //     //                 copy_bidirectional(&mut response_upgraded, &mut request_upgraded).await
    //     //             {
    //     //                 tracing::error!(error = ?e, "coping between upgraded connections failed");
    //     //             }
    //     //         }
    //     //         Err(e) => {
    //     //             tracing::error!(error = ?e, "upgrade request failed");
    //     //         }
    //     //     }
    //     // });
    // }
}

#[async_trait]
impl<U> Handler for PlexProxy<U>
where
    U: Upstreams,
    U::Error: Into<BoxedError>,
{
    #[inline]
    async fn handle(
        &self,
        req: &mut Request,
        _depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        // self.websocket(req, res).await
        self.proxy.handle(req, _depot, res, ctrl).await
    }
}

#[inline]
fn get_upgrade_type(headers: &HeaderMap) -> Option<&str> {
    if headers
        .get(&CONNECTION)
        .map(|value| {
            value
                .to_str()
                .unwrap()
                .split(',')
                .any(|e| e.trim() == UPGRADE)
        })
        .unwrap_or(false)
    {
        if let Some(upgrade_value) = headers.get(&UPGRADE) {
            tracing::debug!(
                "Found upgrade header with value: {:?}",
                upgrade_value.to_str()
            );
            return upgrade_value.to_str().ok();
        }
    }

    None
}
