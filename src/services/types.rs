use futures::future::Future;

use super::error::Error;

/// Service layer Future
pub type ServiceFuture<T> = Box<Future<Item = T, Error = Error>>;
