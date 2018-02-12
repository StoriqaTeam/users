use futures::future::Future;

use super::error::ServiceError;

/// Service layer Future
pub type ServiceFuture<T> = Box<Future<Item = T, Error = ServiceError>>;
