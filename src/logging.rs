//! Logging middleware
use std::time::Instant;

use tracing::{Instrument, Level};

use salvo::http::{Request, Response, StatusCode};
use salvo::{async_trait, Depot, FlowCtrl, Handler};

/// A simple logger middleware.
#[derive(Default, Debug)]
pub struct Logger {}
impl Logger {
    /// Create new `Logger` middleware.
    #[inline]
    pub fn new() -> Self {
        Logger {}
    }
}

#[async_trait]
impl Handler for Logger {
    async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
        let headers = req.headers_mut().clone();
        let span = tracing::span!(
            Level::TRACE,
            "Request",
            remote_addr = %req.remote_addr().to_string(),
            version = ?req.version(),
            method = %req.method(),
            headers = ?headers,
            path = %req.uri(),
            span.kind = "server",
            service.name = "replex",
            name = tracing::field::Empty,
            otel.status_code = tracing::field::Empty,
            otel.status_description = tracing::field::Empty,
        );

        async move {
            let now = Instant::now();
            ctrl.call_next(req, depot, res).await;
            let duration = now.elapsed();
            let status = res.status_code.unwrap_or(StatusCode::OK);
            tracing::debug!(
                status = %status,
                path = %req.uri(),
                duration = ?duration,
                "Response"
            );
        }
        .instrument(span)
        .await
    }
}