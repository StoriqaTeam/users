use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use failure::Error as FailureError;
use futures::future::Future;
use r2d2;

/// Repos layer Future
pub type RepoFuture<T> = Box<Future<Item = T, Error = FailureError>>;
pub type RepoResult<T> = Result<T, FailureError>;
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;
