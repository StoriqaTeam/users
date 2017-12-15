use rocket::http::RawStr;
use cache::connection::RedisConnection;
use redis::Commands;

//
// $ curl -X POST http://localhost:3000/items/first
// OK
// $ curl -X POST http://localhost:3000/items/second
// OK
//
#[post("/<item>")]
fn create(item: &RawStr, conn: RedisConnection) -> String {
    let _: () = conn.client.lpush(conn.db, item.as_str()).unwrap();
    format!("OK")
}

//
// $ curl http://localhost:3000/items
// second, first
//
#[get("/")]
fn index(conn: RedisConnection) -> String {
    let items: Vec<String> = conn.client.lrange(conn.db, 0, -1).unwrap();

    items.join(", ")
}
