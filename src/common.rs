extern crate futures;
extern crate hyper;
extern crate r2d2;

use futures::Future;
use hyper::server::{Request, Response};
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;

pub type TheRequest = Request;
pub type TheResponse = Response;
pub type TheError = hyper::Error;
pub type TheFuture = Box<Future<Item = TheResponse, Error = TheError>>;

pub type ThePool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type TheConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub const MAX_USER_COUNT: i64 = 50;
