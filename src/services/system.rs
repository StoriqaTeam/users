use futures::future;
use futures::Future;

use error::Error as ApiError;
use responses::status::StatusMessage;

/// System service, responsible for common endpoints like healthcheck
pub struct SystemService;

impl SystemService {
    /// Healthcheck endpoint, always returns OK status
    pub fn healthcheck(&self) -> Box<Future<Item = StatusMessage, Error = ApiError>> {
        let message = StatusMessage::new("OK");
        Box::new(future::ok(message))
    }
}
