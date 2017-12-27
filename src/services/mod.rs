pub mod users;
pub mod system;

use futures::future;
use common::TheFuture;
use error::Error as ApiError;
use utils::http::*;

/// Generic service trait
pub trait Service {
    fn respond_with(&self, result: Result<String, ApiError>) -> TheFuture {
        match result {
            Ok(response) => Box::new(future::ok(response_with_json(response))),
            Err(err) => Box::new(future::ok(response_with_error(err)))
        }
    }
}
