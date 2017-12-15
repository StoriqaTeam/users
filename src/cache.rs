use rocket::http;
use rocket::request;
use rocket::Outcome;
use rocket::State;
use r2d2;
use r2d2_redis::RedisConnectionManager;

pub struct RedisConf {
    pub dsn: String,
    pub db: String,
}

// Pool initiation.
// Call it starting an app and store a pul as a rocket managed state.
pub fn pool(cfg: RedisConf) -> RedisPool {
    let manager = RedisConnectionManager::new(cfg.dsn.as_ref()).expect("connection manager");
    let pool = r2d2::Pool::new(manager).expect("db pool");
    RedisPool::new(pool, cfg)
}

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
    pub cfg: RedisConf,
}

impl RedisConnection {
    fn new(c: r2d2::PooledConnection<RedisConnectionManager>, cfg: RedisConf) -> RedisConnection {
        RedisConnection {
            client: c,
            cfg: cfg,
        }
    }
}

// An alias to the type for a pool of redis connections.
type Pool = r2d2::Pool<RedisConnectionManager>;

pub struct RedisPool {
    pool: Pool,
    cfg: RedisConf,
}

impl RedisPool {
    fn new(pool: Pool, cfg: RedisConf) -> RedisPool {
        RedisPool {
            pool: pool,
            cfg: cfg,
        }
    }
}

// Retrieving a single connection from the managed database pool.
impl<'a, 'r> request::FromRequest<'a, 'r> for RedisConnection {
    type Error = ();

    fn from_request(request: &'a request::Request<'r>) -> request::Outcome<RedisConnection, ()> {
        let redis_pool = request.guard::<State<RedisPool>>()?;

        match redis_pool.pool.get() {
            Ok(conn) => Outcome::Success(RedisConnection::new(conn, redis_pool.cfg)),
            Err(_) => Outcome::Failure((http::Status::ServiceUnavailable, ())),
        }
    }
}
