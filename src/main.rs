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

use std::collections::HashMap;
use std::sync::Mutex;

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
}

fn main() {
    rocket().launch();
}
