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
use futures::{Future, Stream};
use hyper::{Get, Post, Put, Delete};
use hyper::server::{Http, Service, Request, Response};
use diesel::prelude::*;
use diesel::pg::PgConnection;

use http_utils::*;
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
            // GET /healthcheck
            (&Get, Some(router::Route::Healthcheck)) => {
                info!("Handling request GET /healthcheck");

                let response = serde_json::to_string(&status_ok()).unwrap();
                Box::new(future::ok(response_with_body(response)))
            },
            // POST /users
            (&Post, Some(router::Route::UsersNew)) => {
                info!("Handling request POST /users");

                Box::new(req.body()
                    .fold(Vec::new(), |mut acc, chunk| {
                        acc.extend_from_slice(&*chunk);
                        futures::future::ok::<_, Self::Error>(acc)
                    })
                    .and_then(|v| {
                        let new_user: NewUser = serde_json::from_slice::<NewUser>(&v).unwrap();

                        let connection = establish_connection();

                        let user = diesel::insert_into(users)
                            .values(&new_user)
                            .get_result::<User>(&connection)
                            .expect("Error saving new user");

                        Ok::<_, Self::Error>(user)
                    }).and_then(|user| {
                        let response = serde_json::to_string(&user).unwrap();
                        future::ok(response_with_body(response))
                    }))
            },
            // PUT /users/1
            (&Put, Some(router::Route::Users(user_id))) => {
                info!("Handling request PUT /users/{}", user_id);

                Box::new(req.body()
                    .fold(Vec::new(), |mut acc, chunk| {
                        acc.extend_from_slice(&*chunk);
                        futures::future::ok::<_, Self::Error>(acc)
                    })
                    .and_then(move|v| {
                        let new_user: UpdateUser = serde_json::from_slice::<UpdateUser>(&v).unwrap();

                        let connection = establish_connection();

                        let user = diesel::update(users.find(user_id))
                            .set(email.eq(new_user.email))
                            .get_result::<User>(&connection)
                            .expect("Error saving new user");

                        Ok::<_, Self::Error>(user)
                    }).and_then(|user| {
                    let response = serde_json::to_string(&user).unwrap();
                    future::ok(response_with_body(response))
                }))
            },
            // GET /users/<user_id>
            (&Get, Some(router::Route::Users(user_id))) => {
                info!("Handling request GET /users/{}", user_id);

                let connection = establish_connection();

                let user = users
                    .find(user_id)
                    .get_result::<User>(&connection)
                    .expect("Error loading user");

                let response = serde_json::to_string(&user).unwrap();
                Box::new(future::ok(response_with_body(response)))
            },
            // GET /users/<from>/<count>
            (&Get, Some(router::Route::UsersList(from, count))) => {
                info!("Handling request GET /users/{}/{}", from, count);

                let connection = establish_connection();

                let results = users
                    .filter(is_active.eq(true))
                    .order(id)
                    .limit(count)
                    .offset(from)
                    .load::<User>(&connection)
                    .expect("Error loading users");

                let response = serde_json::to_string(&results).unwrap();
                Box::new(future::ok(response_with_body(response)))
            },
            // DELETE /users/<user_id>
            (&Delete, Some(router::Route::Users(user_id))) => {
                info!("Handling request DELETE /users/{}", user_id);

                let connection = establish_connection();

                diesel::update(users.find(user_id))
                    .set(is_active.eq(false))
                    .get_result::<User>(&connection)
                    .expect("Error loading user");

                let response = serde_json::to_string(&status_ok()).unwrap();
                Box::new(future::ok(response_with_body(response)))
            },
            // Fallback
            _ => Box::new(future::ok(response_not_found()))
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

    info!("Listening on http://{}", server.local_addr().unwrap());
    server.run().unwrap();
}
