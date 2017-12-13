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
mod db;
mod items;

use rocket::fairing::AdHoc;
use std::collections::HashMap;
use std::sync::Mutex;

const REDIS_ADDRESS: &'static str = "redis://localhost:6379";

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![system::healthcheck])
        .mount("/items", routes![items::create, items::index])
        .mount(
            "/message",
            routes![message::new, message::update, message::get],
        )
        .catch(errors![system::not_found, system::bad_request])
        .manage(db::pool())
        .manage(Mutex::new(HashMap::<message::ID, String>::new()))
        .attach(AdHoc::on_attach(|rocket| {
            println!("Adding redis DSN to managed state...");
            let redis_dsn = rocket.config().get_str("redis").unwrap_or(REDIS_ADDRESS);
            Ok(rocket.manage(redis_dsn))
        }))
}

fn main() {
    rocket().launch();
}
