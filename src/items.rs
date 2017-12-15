use rocket::http::RawStr;
use cache::connection::RedisConnection;
use redis::Commands;

//
// $ curl -X POST http://localhost:8000/first
// OK
// $ curl -X POST http://localhost:8000/second
// OK
//
#[post("/<item>")]
fn create(item: &RawStr, conn: RedisConnection) -> String {
    let _: () = conn.client.lpush(conn.cfg.db, item.as_str()).unwrap();
    format!("OK")
}

//
// $ curl http://localhost:8000
// second, first
//
#[get("/")]
fn index(conn: RedisConnection) -> String {
    let items: Vec<String> = conn.client.lrange(conn.cfg.db, 0, -1).unwrap();

    items.join(", ")
}
