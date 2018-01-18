use super::error::Error;

pub type ControllerFuture = Box<Future<Item = String, Error = Error>>;
