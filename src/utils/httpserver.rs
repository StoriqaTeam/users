use hyper::{StatusCode};
use hyper::mime;
use hyper::header::{ContentLength, ContentType};
use hyper::server::{Request, Response};
use hyper::error::Error;

use futures::future::{Future};
use futures::{future, Stream};

use std::collections::HashMap;
use std::iter::FromIterator;

use hyper;
use error;

/// Splits query string to key-value pairs
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

/// Reads request body and returns it in a Future
pub fn read_body(request: Request) -> Box<Future<Item=String, Error=hyper::Error>> {
    Box::new(
        request.body()
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
