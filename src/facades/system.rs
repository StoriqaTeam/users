use std::sync::Arc;

use serde_json;
use futures::future;
use futures::Future;

use common::TheFuture;
use error::Error as ApiError;
use services::system::SystemService;
use utils::httpserver::*;

pub struct SystemFacade {
    pub system_service: Arc<SystemService>
}

impl SystemFacade {
    pub fn healthcheck(&self) -> TheFuture {
        let future = self.system_service.healthcheck()
            .and_then(|message| {
                serde_json::to_string(&message).map_err(|e| ApiError::from(e))
            })
            .then(|res| match res {
                Ok(data) => future::ok(response_with_json(data)),
                Err(err) => future::ok(response_with_error(err))
            });

        Box::new(future)
    }
}
