use futures::future::Future;
use super::error::Error;

pub type RepoFuture<T> = Box<Future<Item = T, Error = Error>>;
