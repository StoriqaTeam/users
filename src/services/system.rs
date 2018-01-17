use futures::future;

use responses::status::StatusMessage;
use super::types::ServiceFuture;

/// System service, responsible for common endpoints like healthcheck
pub struct SystemService;

impl SystemService {
    /// Healthcheck endpoint, always returns OK status
    pub fn healthcheck(&self) -> ServiceFuture<StatusMessage> {
        let message = StatusMessage::new("OK");
        Box::new(future::ok(message))
    }
}
