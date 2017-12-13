#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

mod system;
mod message;

use std::collections::HashMap;
use std::sync::Mutex;

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![system::healthcheck])
        .mount(
            "/message",
            routes![message::new, message::update, message::get],
        )
        .catch(errors![system::not_found, system::bad_request])
        .manage(Mutex::new(HashMap::<message::ID, String>::new()))
}

fn main() {
    rocket().launch();
}
