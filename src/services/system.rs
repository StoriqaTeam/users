use futures::future;

use super::types::ServiceFuture;

/// System service, responsible for common endpoints like healthcheck
pub struct SystemService;

impl SystemService {
    /// Healthcheck endpoint, always returns OK status
    pub fn healthcheck(&self) -> ServiceFuture<String> {
        Box::new(future::ok("Ok".to_string()))
    }
}
