
use futures::Future;
use hyper;
use super::error::Error;

pub type ServerFuture = Box<Future<Item = hyper::Response, Error = hyper::Error>>;
