use hyper::http;
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
use std::net::SocketAddr;
use tokio::io::copy_bidirectional;
use tracing::debug;
use url::Url;

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::client::conn::http1::Builder;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper::{Method, Request, Response};

use tokio::net::{TcpListener, TcpStream};

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
        req: &mut salvo::Request,
        _depot: &mut Depot,
        res: &mut salvo::Response,
        ctrl: &mut FlowCtrl,
    ) {
        println!("req: {:?}", req);

        let mut hyper_req = hyper::Request::builder()
            .uri(req.uri())
            // .header("X-Plex-Client-Identifier", &self.x_plex_client_identifier)
            // .header("X-Plex-Token", &self.x_plex_token)
            // // .header("Accept", &self.content_type.to_string())
            // .header("Accept", "application/json")
            .method(req.method().clone())
            .body(ReqBody::None)
            .map_err(Error::other)
            .unwrap();

        *hyper_req.headers_mut() = req.headers_mut().clone();

        if Method::CONNECT == hyper_req.method() {
            // Received an HTTP request like:
            // ```
            // CONNECT www.domain.com:443 HTTP/1.1
            // Host: www.domain.com:443
            // Proxy-Connection: Keep-Alive
            // ```
            //
            // When HTTP method is CONNECT we should return an empty body
            // then we can eventually upgrade the connection and talk a new protocol.
            //
            // Note: only after client received an empty body with STATUS_OK can the
            // connection be upgraded, so we can't return a response inside
            // `on_upgrade` future.
            if let Some(addr) = host_addr(hyper_req.uri()) {
                tokio::task::spawn(async move {
                    match hyper::upgrade::on(hyper_req).await {
                        Ok(upgraded) => {
                            if let Err(e) = tunnel(upgraded, addr).await {
                                eprintln!("server io error: {}", e);
                            };
                        }
                        Err(e) => eprintln!("upgrade error: {}", e),
                    }
                });

                //Ok(Response::new(empty()))
            } else {
                eprintln!(
                    "CONNECT host is not socket addr: {:?}",
                    hyper_req.uri()
                );
                let mut resp =
                    Response::new(full("CONNECT must be to a socket address"));
                *resp.status_mut() = http::StatusCode::BAD_REQUEST;

                //Ok(resp)
            }
        } else {
            let host = hyper_req.uri().host().expect("uri has no host");
            let port = hyper_req.uri().port_u16().unwrap_or(80);
            let addr = format!("{}:{}", host, port);

            let stream = TcpStream::connect(addr).await.unwrap();
            let io = TokioIo::new(stream);

            let (mut sender, conn) = Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .handshake(io)
                .await
                .unwrap();
            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    println!("Connection failed: {:?}", err);
                }
            });

            let mut resp = sender.send_request(hyper_req).await.unwrap();
            //res = resp;
            //Ok(resp.map(|b| b.boxed()))
        }
    }

    // #[inline]
    // async fn handle(
    //     &self,
    //     req: &mut Request,
    //     _depot: &mut Depot,
    //     res: &mut Response,
    //     ctrl: &mut FlowCtrl,
    // ) {
    //     let url = Url::parse(&req.uri_mut().to_string()).unwrap();
    //     let mut reqqq = reqwest::Request::new(
    //         req.method().clone(),
    //         url,
    //     );

    //     *reqqq.headers_mut() = req.headers().clone();
    //     *reqqq.version_mut() = req.version().clone();

    //     // let reqq = reqwest::RequestBuilder.;
    //     let ress = self.client.execute(reqqq).await;
    //     // dbg!(&ress);

    //     // salvo::Response {
    //     //     status_code: None,
    //     //     body: salvo::ResBody::None,
    //     //     version: salvo::Version::default(),
    //     //     headers: salvo::HeaderMap::new(),
    //     //     #[cfg(feature = "cookie")]
    //     //     cookies: salvo::CookieJar::default(),
    //     //     extensions: salvo::Extensions::new(),
    //     // }
    // }
}

fn host_addr(uri: &http::Uri) -> Option<String> {
    uri.authority().and_then(|auth| Some(auth.to_string()))
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

// Create a TCP connection to host:port, build a tunnel between the connection and
// the upgraded connection
async fn tunnel(upgraded: Upgraded, addr: String) -> std::io::Result<()> {
    // Connect to remote server
    let mut server = TcpStream::connect(addr).await?;
    let mut upgraded = TokioIo::new(upgraded);

    // Proxying data
    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    // Print message when done
    println!(
        "client wrote {} bytes and received {} bytes",
        from_client, from_server
    );

    Ok(())
}
