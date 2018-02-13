use futures::future;

use super::types::ServiceFuture;

/// System service, responsible for common endpoints like healthcheck
pub trait SystemService {
    /// Healthcheck endpoint, always returns OK status
    fn healthcheck(&self) -> ServiceFuture<String>;
}

pub struct SystemServiceImpl;

impl SystemService for SystemServiceImpl {
    /// Healthcheck endpoint, always returns OK status
    fn healthcheck(&self) -> ServiceFuture<String> {
        Box::new(future::ok("Ok".to_string()))
    }
}

impl SystemServiceImpl {
    pub fn new() -> Self {
        Self {}
    }
}
