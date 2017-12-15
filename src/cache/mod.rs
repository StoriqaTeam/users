pub mod pool;
pub mod connection;

pub struct RedisConf {
    pub dsn: String,
    pub db: String,
}
