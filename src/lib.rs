extern crate dotenv;
extern crate futures;
extern crate hyper;
extern crate regex;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate diesel;

pub mod error;
pub mod router;
pub mod http_utils;
pub mod schema;
pub mod models;

use std::env;
use std::sync::Arc;

use dotenv::dotenv;
use futures::future;
use hyper::Get;
use hyper::server::{Http, Service, Request, Response};
use diesel::prelude::*;
use diesel::pg::PgConnection;

use models::*;
use schema::users::dsl::*;

struct WebService {
    router: Arc<router::Router>
}

impl Service for WebService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<futures::Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        match (req.method(), self.router.test(req.path())) {
            (&Get, Some(router::Route::Root)) => {
                let connection = establish_connection();

                let results = users
                    .filter(is_active.eq(true))
                    .limit(5)
                    .load::<User>(&connection)
                    .expect("Error loading users");

                let serialized = serde_json::to_string(&results).unwrap();
                Box::new(future::ok(http_utils::response_with_body(serialized)))
            },
            (&Get, Some(router::Route::Users(user_id))) =>
                Box::new(future::ok(http_utils::response_with_body(user_id.to_string()))),

            _ => Box::new(future::ok(http_utils::response_not_found()))
        }
    }
}

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn start_server() {
    dotenv().ok();
    env_logger::init().unwrap();

    let address = env::var("HTTP_ADDR").expect("HTTP_ADDR must be set");
    let port = env::var("HTTP_PORT").expect("HTTP_PORT must be set");
    let bind = format!("{}:{}", address, port);

    let addr = match bind.parse() {
        Result::Ok(val) => val,
        Result::Err(err) => panic!("Error: {}", err),
    };

    let mut server = Http::new().bind(&addr, || {
        let service = WebService {
            router: Arc::new(router::create_router())
        };
        Ok(service)
    }).unwrap();

    server.no_proto();

    info!("Listening on http://{}.", server.local_addr().unwrap());
    server.run().unwrap();
}
