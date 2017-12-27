use serde_json;
use common::TheFuture;
use responses::status::StatusMessage;
use services::Service;

/// System service, responsible for common endpoints like healthcheck
pub struct SystemService;

impl Service for SystemService {}

impl SystemService {
    /// Healthcheck endpoint, always returns OK status
    pub fn healthcheck(&self) -> TheFuture {
        let message = StatusMessage::new("OK");
        let response = serde_json::to_string(&message).unwrap();
        self.respond_with(Ok(response))
    }
}
