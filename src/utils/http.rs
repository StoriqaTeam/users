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

macro_rules! get_and_parse {
    ($hash:expr, $t: ty, $key: tt) => ($hash.get($key).and_then(|value| value.parse::<$t>().ok()))
}

#[macro_export]
macro_rules! params {
    ($query: expr, $e:tt -> $t:ty) => ({ let hash = $crate::utils::http::query_params($query); get_and_parse!(hash, $t, $e) });
    ($query: expr, $e1:tt -> $t1:ty, $e2:tt -> $t2:ty) => ({ let hash = $crate::utils::http::query_params($query); (get_and_parse!(hash, $t1, $e1), get_and_parse!(hash, $t2, $e2)) });
    ($query: expr, $e1:tt -> $t1:ty, $e2:tt -> $t2:ty, $e3:tt -> $t3:ty) => ({ let hash = $crate::utils::http::query_params($query); (get_and_parse!(hash, $t1, $e1), get_and_parse!(hash, $t2, $e2), get_and_parse!(hash, $t3, $e3)) });
    ($query: expr, $e1:tt -> $t1:ty, $e2:tt -> $t2:ty, $e3:tt -> $t3:ty, $e4:tt -> $t4:ty) => ({ let hash = $crate::utils::http::query_params($query); (get_and_parse!(hash, $t1, $e1), get_and_parse!(hash, $t2, $e2), get_and_parse!(hash, $t3, $e3), get_and_parse!(hash, $t4, $e4)) });
    ($query: expr, $e1:tt -> $t1:ty, $e2:tt -> $t2:ty, $e3:tt -> $t3:ty, $e4:tt -> $t4:ty, $e5:tt -> $t5:ty) => ({ let hash = $crate::utils::http::query_params($query); (get_and_parse!(hash, $t1, $e1), get_and_parse!(hash, $t2, $e2), get_and_parse!(hash, $t3, $e3), get_and_parse!(hash, $t4, $e4), get_and_parse!(hash, $t5, $e5)) });
}

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

pub fn parse_body<T>(req: Request) -> Box<Future<Item=T, Error=error::Error>>
    where
        T: for<'a> Deserialize<'a> + Validate + 'static
{
    Box::new(
        read_body(req)
            .map_err(|err| error::Error::from(err))
            .and_then(|body| serde_json::from_str::<T>(&body).map_err(|_| error::Error::UnprocessableEntity))
            .and_then(|payload| match payload.validate() {
                Ok(_) => Ok(payload),
                Err(e) => Err(error::Error::from(e))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn params_1() {
        assert_eq!(params!("from=12", "from" -> i32), Some(12));
        assert_eq!(params!("from=12a", "from" -> i32), None);
        assert_eq!(params!("from=12", "to" -> i32), None);
    }

    #[test]
    fn params_2() {
        assert_eq!(params!("from=12&to=22", "from" -> i32, "to" -> i64), (Some(12), Some(22)));
        assert_eq!(params!("from=12&to=22", "from" -> i32, "to" -> String), (Some(12), Some("22".to_string())));
        assert_eq!(params!("from=12&to=true", "from" -> bool, "to" -> bool), (None, Some(true)));
    }

    #[test]
    fn params_3() {
        assert_eq!(params!("from=12&to=22&published=true", "from" -> i32, "to" -> i64, "published" -> bool), (Some(12), Some(22), Some(true)));
    }

    #[test]
    fn params_4() {
        assert_eq!(params!("from=12&to=22&published=true&name=Alex", "from" -> i32, "to" -> i64, "published" -> bool, "name" -> String), (Some(12), Some(22), Some(true), Some("Alex".to_string())));
    }

    #[test]
    fn params_5() {
        assert_eq!(params!("from=12&to=22&published=true&name=Alex&price=3.25", "from" -> i32, "to" -> i64, "published" -> bool, "name" -> String, "price" -> f32), (Some(12), Some(22), Some(true), Some("Alex".to_string()), Some(3.25)));
    }
}