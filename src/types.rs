use futures::Future;
use hyper;

pub type ServerFuture = Box<Future<Item = hyper::Response, Error = hyper::Error>>;
