use futures::future::Future;
use super::error::Error;

/// Repos layer Future
pub type RepoFuture<T> = Box<Future<Item = T, Error = Error>>;
