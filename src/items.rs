use rocket::http::RawStr;
use cache::connection::RedisConnection;
use r2d2;
use r2d2_redis::RedisConnectionManager;
use std::ops::Deref;
use redis::Commands;

impl Deref for RedisConnection {
    type Target = r2d2::PooledConnection<RedisConnectionManager>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

//
// $ curl -X POST http://localhost:8000/first
// OK
// $ curl -X POST http://localhost:8000/second
// OK
//
#[post("/<item>")]
fn create(item: &RawStr, conn: RedisConnection) -> String {
    let _: () = conn.lpush(conn.cfg.db, item.as_str()).unwrap();
    format!("OK")
}

//
// $ curl http://localhost:8000
// second, first
//
#[get("/")]
fn index(conn: RedisConnection) -> String {
    let items: Vec<String> = conn.lrange(conn.cfg.db, 0, -1).unwrap();

    items.join(", ")
}
