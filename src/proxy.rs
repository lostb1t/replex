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
use salvo::test::ResponseExt;
use tokio::io::copy_bidirectional;
use tracing::debug;
use url::Url;

type HyperRequest = hyper::Request<ReqBody>;
type HyperResponse = hyper::Response<ResBody>;

pub struct PlexProxy {
    pub client: reqwest::Client,
}

impl PlexProxy {
    pub fn new(upstream: String) -> Self {
        PlexProxy {
            client: reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(upstream).unwrap())
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl Handler for PlexProxy {
    #[inline]
    async fn handle(
        &self,
        req: &mut Request,
        _depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        let url = Url::parse(&req.uri_mut().to_string()).unwrap();
        let mut reqqq = reqwest::Request::new(
            req.method().clone(),
            url,
        );

        *reqqq.headers_mut() = req.headers().clone();
        *reqqq.version_mut() = req.version().clone();

        // let reqq = reqwest::RequestBuilder.;
        let ress = self.client.execute(reqqq).await;
        dbg!(&ress);
    }
}
