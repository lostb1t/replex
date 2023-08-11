//! basic auth middleware
use std::time::Duration;

use salvo::http::{Request, Response, StatusError};
use salvo::{async_trait, Depot, FlowCtrl, Handler};

/// Timeout
pub struct Timeout {
    value: Duration,
}
impl Timeout {
    /// Create a new `Timeout`.
    #[inline]
    pub fn new(value: Duration) -> Self {
        Timeout { value }
    }
}
#[async_trait]
impl Handler for Timeout {
    #[inline]
    async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
        tokio::select! {
            _ = ctrl.call_next(req, depot, res) => {},
            _ = tokio::time::sleep(self.value) => {
                res.render(StatusError::internal_server_error().brief("Server process the request timeout."));
                ctrl.skip_rest();
            }
        }
    }
}