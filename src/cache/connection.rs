use rocket::http;
use rocket::request;
use rocket::Outcome;
use rocket::State;
use r2d2;
use r2d2_redis::RedisConnectionManager;

use cache::pool::RedisPool;

// Rocket guard type: a wrapper around an r2d2 pool.
// In conjunction with
// `impl<'a, 'r> request::FromRequest<'a, 'r> for RedisConnection` (see below)
// it allows code like:
//   ```
//   #[post("/<item>")]
//   fn create(item: &RawStr, connection: RedisConnection) -> ...
//
pub struct RedisConnection {
    pub client: r2d2::PooledConnection<RedisConnectionManager>,
    pub db: String,
}

impl RedisConnection {
    fn new(c: r2d2::PooledConnection<RedisConnectionManager>, db: &str) -> RedisConnection {
        RedisConnection {
            client: c,
            db: String::from(db),
        }
    }
}

// Retrieving a single connection from the managed database pool.
impl<'a, 'r> request::FromRequest<'a, 'r> for RedisConnection {
    type Error = ();

    fn from_request(request: &'a request::Request<'r>) -> request::Outcome<RedisConnection, ()> {
        let redis_pool = request.guard::<State<RedisPool>>()?;

        match redis_pool.pool.get() {
            Ok(conn) => Outcome::Success(RedisConnection::new(conn, redis_pool.get_db())),
            Err(_) => Outcome::Failure((http::Status::ServiceUnavailable, ())),
        }
    }
}
