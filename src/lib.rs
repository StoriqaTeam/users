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

                Box::new(
                    read_body(req)
                        .and_then(move |body| {
                            let result = (serde_json::from_slice::<NewUser>(&body.as_bytes()) as Result<NewUser, serde_json::error::Error>)
                                .and_then(|new_user| {
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

                let conn = self.get_connection();

                let user = users
                    .find(user_id)
                    .get_result::<User>(&*conn)
                    .expect("Error loading user");

                let response = serde_json::to_string(&user).unwrap();
                Box::new(future::ok(response_with_body(response)))
            },
            // GET /users
            (&Get, Some(router::Route::Users)) => {
                info!("Handling request GET /users");

                let query = req.uri().query().unwrap();
                info!("Query: {}", query);

                let query_params = query_params(query);
                let from = query_params.get("from").and_then(|v| v.parse::<i32>().ok()).expect("From value");
                let count = query_params.get("count").and_then(|v| v.parse::<i64>().ok()).expect("Count value");

                let conn = self.get_connection();

                let results = users
                    .filter(is_active.eq(true))
                    .filter(id.gt(from))
                    .order(id)
                    .limit(count)
                    .load::<User>(&*conn)
                    .expect("Error loading users");

                let response = serde_json::to_string(&results).unwrap();
                Box::new(future::ok(response_with_body(response)))
            },
            // DELETE /users/<user_id>
            (&Delete, Some(router::Route::User(user_id))) => {
                info!("Handling request DELETE /users/{}", user_id);

                let conn = self.get_connection();

                diesel::update(users.find(user_id))
                    .set(is_active.eq(false))
                    .get_result::<User>(&*conn)
                    .expect("Error loading user");

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
    let addr = settings.http.bind.parse().expect("Bind address must be set in configuration");
    let mut server = Http::new().bind(&addr, move || {
        // Prepare database pool
        let database_url: String = settings.database.dsn.parse().expect("Database DSN must be set in configuration");
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
