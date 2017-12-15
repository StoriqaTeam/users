use r2d2;
use r2d2_redis::RedisConnectionManager;
use cache::RedisConf;

// An alias to the type for a pool of redis connections.
type Pool = r2d2::Pool<RedisConnectionManager>;

pub struct RedisPool {
    pub pool: Pool,
    pub cfg: RedisConf,
}

impl RedisPool {
    pub fn new(cfg: RedisConf) -> RedisPool {
        let manager = RedisConnectionManager::new(cfg.dsn.as_ref()).expect("connection manager");
        let pool = r2d2::Pool::new(manager).expect("db pool");

        RedisPool {
            pool: pool,
            cfg: cfg,
        }
    }
}
