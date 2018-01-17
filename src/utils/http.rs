use std::collections::HashMap;
use std::iter::FromIterator;

use hyper::{StatusCode};
use hyper::mime;
use hyper::header::{ContentLength, ContentType};
use hyper::server::{Request, Response};
use hyper::error::Error;
use futures::future::{Future};
use futures::{future, Stream};
use serde_json;
use serde::de::Deserialize;
use validator::Validate;

use hyper;
use error;

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
        T: for<'a> Deserialize<'a> + Validate + 'static
{
    Box::new(
        read_body(req.body())
            .map_err(|err| error::Error::from(err))
            .and_then(|body| serde_json::from_str::<T>(&body).map_err(|_| error::Error::UnprocessableEntity))
            .and_then(|payload| match payload.validate() {
                Ok(_) => Ok(payload),
                Err(e) => Err(error::Error::from(e))
            })
    )
}

/// Reads body of request and response in Future format
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
                    Err(err) => future::err(Error::Utf8(err.utf8_error()))
                }
            })
    )
 }


fn response_with_body(body: String) -> Response {
    Response::new()
        .with_header(ContentLength(body.len() as u64))
        .with_header(ContentType(mime::APPLICATION_JSON))
        .with_status(StatusCode::Ok)
        .with_body(body)
}

/// Responds with JSON, logs response body
pub fn response_with_json(body: String) -> Response {
    info!("{}", body);
    response_with_body(body)
}

/// Responds with JSON error, logs response body
pub fn response_with_error(error: error::Error) -> Response {
    error!("{}", error.to_json());
    response_with_body(error.to_json()).with_status(error.to_code())
}

/// Responds with 'not found' JSON error and status code
pub fn response_not_found() -> Response {
    response_with_body("Not found".to_string()).with_status(StatusCode::NotFound)
}