use futures::future::Future;

use super::error::Error;

pub type ServiceFuture<T> = Box<Future<Item = T, Error = Error>>;
