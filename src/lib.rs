extern crate config;
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
extern crate r2d2;
extern crate r2d2_diesel;

pub mod error;
pub mod router;
pub mod http_utils;
pub mod schema;
pub mod models;
pub mod settings;

use std::sync::Arc;

use futures::future;
use futures::Future;
use hyper::{Get, Post, Put, Delete};
use hyper::server::{Http, Service, Request, Response};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;

use http_utils::*;
use models::*;
use schema::users::dsl::*;
use settings::Settings;

type ThePool = r2d2::Pool<ConnectionManager<PgConnection>>;
type TheConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

const MAX_USER_COUNT: i64 = 50;

struct WebService {
    router: Arc<router::Router>,
    pool: Arc<ThePool>
}

impl WebService {
    fn get_connection(&self) -> TheConnection {
        match self.pool.get() {
            Ok(connection) => connection,
            Err(e) => panic!("Error obtaining connection from pool: {}", e)
        }
    }

    fn error(&self, error: error::Error) -> <WebService as Service>::Future {
        Box::new(future::ok(response_with_error(error)))
    }

    fn json_error(&self, message: &str) -> <WebService as Service>::Future {
        let error = error::Error::new(message);
        Box::new(future::ok(response_with_error(error)))
    }
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
            (&Post, Some(router::Route::Users)) => {
                info!("Handling request POST /users");
                let conn = self.get_connection();

                read_body(req).map(move |body| {
                    let result = serde_json::from_slice::<NewUser>(&body.as_bytes());
//                    let new_user = match result {
//                        Ok(user) => user,
//                        Err(_) => {}
//                    };

                    response_with_body(data)
                })

                /*
                Box::new(
                    read_body(req)
                        .and_then(move |body| {
                            let result = (serde_json::from_slice::<NewUser>(&body.as_bytes()) as Result<NewUser, serde_json::error::Error>)
                                .and_then(|new_user| {
                                    // Insert user
                                    let user = diesel::insert_into(users)
                                        .values(&new_user)
                                        .get_result::<User>(&*conn)
                                        .expect("Error saving new user");

                                    let response = serde_json::to_string(&user).unwrap();
                                    Ok::<_, serde_json::Error>(response)
                                });

                            match result {
                                Ok(data) => future::ok(response_with_body(data)),
                                Err(err) => future::ok(response_with_error(error::Error::Json(err)))
                            }
                        })
                )
                */
            },
            // PUT /users/1
            (&Put, Some(router::Route::User(user_id))) => {
                info!("Handling request PUT /users/{}", user_id);

                let conn = self.get_connection();

                Box::new(
                    read_body(req)
                        .and_then(move |body| {
                            let result = (serde_json::from_slice::<UpdateUser>(&body.as_bytes()) as Result<UpdateUser, serde_json::error::Error>)
                                .and_then(|new_user| {
                                    // Update user
                                    let user = diesel::update(users.find(user_id))
                                        .set(email.eq(new_user.email))
                                        .get_result::<User>(&*conn)
                                        .expect("Error updating user");

                                    let response = serde_json::to_string(&user).unwrap();
                                    Ok::<_, serde_json::Error>(response)
                                });

                            match result {
                                Ok(data) => future::ok(response_with_body(data)),
                                Err(err) => future::ok(response_with_error(error::Error::Json(err)))
                            }
                        })
                )
            },
            // GET /users/<user_id>
            (&Get, Some(router::Route::User(user_id))) => {
                info!("Handling request GET /users/{}", user_id);

                // Get user from database
                let conn = self.get_connection();
                let result = users.find(user_id).get_result::<User>(&*conn);

                let result = match result {
                    Ok(user) => user,
                    Err(_) => return self.json_error("User not found")
                };

                let response = match serde_json::to_string(&result) {
                    Ok(response) => response,
                    Err(err) => return self.error(error::Error::Json(err))
                };

                Box::new(future::ok(response_with_body(response)))
            },
            // GET /users
            (&Get, Some(router::Route::Users)) => {
                info!("Handling request GET /users");

                // Validate query string
                let query = match req.uri().query() {
                    Some(value) => value,
                    None => return self.json_error("No query parameters provided")
                };

                info!("Query: {}", query);
                let query_params = query_params(query);

                // Validate query parameters
                let from = match query_params.get("from").and_then(|v| v.parse::<i32>().ok()) {
                    Some(value) if value > 0 => value,
                    Some(_) => return self.json_error("`from` value should be greater than zero"),
                    None => return self.json_error("Invalid `from` value provided")
                };

                let count = match query_params.get("count").and_then(|v| v.parse::<i64>().ok()) {
                    Some(value) if value < MAX_USER_COUNT => value,
                    Some(_) => return self.json_error("Too much users requested, try less `count` value"),
                    None => return self.json_error("Invalid `count` value provided")
                };

                // Get users from database
                let conn = self.get_connection();

                let results = users
                    .filter(is_active.eq(true))
                    .filter(id.gt(from))
                    .order(id)
                    .limit(count)
                    .load::<User>(&*conn);

                let result = match results {
                    Ok(list) => list,
                    Err(err) => return self.error(error::Error::Database(err))
                };

                let response = match serde_json::to_string(&result) {
                    Ok(response) => response,
                    Err(err) => return self.error(error::Error::Json(err))
                };

                Box::new(future::ok(response_with_body(response)))
            },
            // DELETE /users/<user_id>
            (&Delete, Some(router::Route::User(user_id))) => {
                info!("Handling request DELETE /users/{}", user_id);

                // Check if user exists
                let conn = self.get_connection();
                let source = users.find(user_id);
                match source.load::<User>(&*conn) {
                    Ok(_) => {},
                    Err(err) => return self.error(error::Error::Database(err))
                }

                // Update
                let updated = diesel::update(source)
                    .set(is_active.eq(false))
                    .get_result::<User>(&*conn);

                match updated {
                    Ok(_) => {},
                    Err(err) => return self.error(error::Error::Database(err))
                };

                let response = serde_json::to_string(&status_ok()).unwrap();
                Box::new(future::ok(response_with_body(response)))
            },
            // Fallback
            _ => Box::new(future::ok(response_not_found()))
        }
    }
}

pub fn start_server(settings: Settings) {
    // Prepare logger
    env_logger::init().unwrap();

    // Prepare server
    let address = settings.address.parse().expect("Address must be set in configuration");
    let mut server = Http::new().bind(&address, move || {
        // Prepare database pool
        let database_url: String = settings.database.parse().expect("Database URL must be set in configuration");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let r2d2_pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create connection pool");

        // Prepare service
        let service = WebService {
            router: Arc::new(router::create_router()),
            pool: Arc::new(r2d2_pool),
        };

        Ok(service)
    }).unwrap();

    server.no_proto();

    // Start
    info!("Listening on http://{}", server.local_addr().unwrap());
    server.run().unwrap();
}
