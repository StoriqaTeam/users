#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_infer_schema;
extern crate dotenv;
extern crate env_logger;
extern crate iron;
#[macro_use]
extern crate log;
extern crate logger;
extern crate router;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use iron::prelude::*;
use iron::{AfterMiddleware, Chain, Iron, IronResult, Request, Response};
use logger::Logger;

pub mod models;
pub mod schema;

use iron::prelude::*;
use iron::status;
use router::Router;

use models::*;
use schema::posts::dsl::*;

struct DefaultContentType;

impl AfterMiddleware for DefaultContentType {
    fn after(&self, _req: &mut Request, mut resp: Response) -> IronResult<Response> {
        if resp.headers.get::<iron::headers::ContentType>() == None {
            resp.headers.set(iron::headers::ContentType::json());
        }
        Ok(resp)
    }
}

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn handler(req: &mut Request) -> IronResult<Response> {
    let connection = establish_connection();
    let results = posts
        .filter(published.eq(true))
        .limit(5)
        .load::<Post>(&connection)
        .expect("Error loading posts");

    let serialized = serde_json::to_string(&results).unwrap();
    Ok(Response::with((status::Ok, serialized)))
}

fn main() {
    dotenv().ok();
    env_logger::init().unwrap();

    let addr = env::var("HTTP_ADDR").expect("HTTP_ADDR must be set");
    let port = env::var("HTTP_PORT").expect("HTTP_PORT must be set");
    let bind = format!("{}:{}", addr, port);

    let mut router = Router::new();
    router.get("/", handler, "index");

    let (logger_before, logger_after) = Logger::new(None);
    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    chain.link_after(DefaultContentType);

    match Iron::new(chain).http(bind) {
        Result::Ok(_) => info!("Listening on {}:{}", addr, port),
        Result::Err(err) => panic!("Error: {}", err),
    }
}
