use std::collections::HashMap;
use std::iter::FromIterator;

use hyper;
use hyper::server::Request;
use futures::future::{Future};
use futures::{future, Stream};
use serde_json;
use serde::de::Deserialize;


use super::error;

/// Splits query string to key-value pairs. See `macros::parse_query` for more sophisticated parsing.
// TODO: Cover more complex cases, e.g. `from=count=10`
pub fn query_params(query: &str) -> HashMap<&str, &str> {
    HashMap::from_iter(
        query.split("&")
            .map(|pair| {
                let mut params = pair.split("=");
                (params.next().unwrap(), params.next().unwrap_or(""))
            })
    )
}

/// Transforms request body with the following pipeline:
///
///   1. Parse request body into entity of type T (T must implement `serde::de::Deserialize` trait)
///
///   2. Validate entity (T must implement `validator::Validate`)
///
/// Fails with `error::Error::UnprocessableEntity` if step 1 fails.
///
/// Fails with `error::Error::BadRequest` with message if step 2 fails.
pub fn parse_body<T>(req: Request) -> Box<Future<Item=T, Error=error::Error>>
    where
        T: for<'a> Deserialize<'a> + 'static
{
    Box::new(
        read_body(req.body())
            .map_err(|err| error::Error::BadRequest(format!("{}", err)))
            .and_then(|body| serde_json::from_str::<T>(&body).map_err(|_| error::Error::UnprocessableEntity("Errpr parsing request body".to_string())))
    )
}


pub fn read_body(body: hyper::Body) -> Box<Future<Item=String, Error=hyper::Error>> {
    Box::new(
        body
            .fold(Vec::new(), |mut acc, chunk| {
                acc.extend_from_slice(&*chunk);
                future::ok::<_, hyper::Error>(acc)
            })
            .and_then(|bytes| {
                match String::from_utf8(bytes) {
                    Ok(data) => future::ok(data),
                    Err(err) => future::err(hyper::Error::Utf8(err.utf8_error()))
                }
            })
    )
 }
