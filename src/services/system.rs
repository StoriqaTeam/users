use serde_json;
use common::TheFuture;
use error::StatusMessage;
use services::Service;

pub struct SystemService;

impl Service for SystemService {}

impl SystemService {
    pub fn healthcheck(&self) -> TheFuture {
        let message = StatusMessage::new("OK");
        let response = serde_json::to_string(&message).unwrap();
        self.respond_with(Ok(response))
    }
}
