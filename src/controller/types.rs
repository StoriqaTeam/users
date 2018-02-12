use futures::future::Future;
use super::error::ControllerError;

pub type ControllerFuture = Box<Future<Item = String, Error = ControllerError>>;
