#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

mod system;
mod message;
mod cache;
mod items;

use rocket::fairing::AdHoc;
use std::collections::HashMap;
use std::sync::Mutex;

fn rocket() -> rocket::Rocket {
    let redis_routes = routes![items::create, items::index];
    let message_routes = routes![message::new, message::update, message::get];
    let error_handlers = errors![system::not_found, system::bad_request];

    rocket::ignite()
        .mount("/", routes![system::healthcheck])
        .mount("/items", redis_routes)
        .mount("/message", message_routes)
        .catch(error_handlers)
        .manage(Mutex::new(HashMap::<message::ID, String>::new()))
        .attach(AdHoc::on_attach(|rocket| {
            let redis_cfg = {
                let redis_dsn = rocket.config().get_str("redis_dsn").unwrap_or("");
                let redis_db = rocket.config().get_str("redis_db").unwrap_or("");
                cache::RedisConfig(redis_dsn.to_string(), redis_db.to_string())
            };

            Ok(rocket.manage(cache::pool(redis_cfg)))
        }))
}

fn main() {
    rocket().launch();
}
