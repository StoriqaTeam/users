extern crate futures;
extern crate hyper;
extern crate r2d2;

use futures::Future;
use hyper::server::{Request, Response};
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;

/// Type alias for Hyper `Request`
pub type TheRequest = Request;

/// Type alias for Hyper `Response`
pub type TheResponse = Response;

/// Type alias for Hyper `Error`
pub type TheError = hyper::Error;

/// Type alias for boxed Future
pub type TheFuture = Box<Future<Item = TheResponse, Error = TheError>>;

/// Type alias for connection pool
pub type ThePool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Type alias for Postgres connection
pub type TheConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

/// Max value of requested users
pub const MAX_USER_COUNT: i64 = 50;
