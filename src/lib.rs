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

use error::Error as ApiError;
use error::StatusMessage;
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

    fn respond_with(&self, result: Result<String, ApiError>) -> <WebService as Service>::Future {
        match result {
            Ok(response) => Box::new(future::ok(response_with_json(response))),
            Err(err) => Box::new(future::ok(response_with_error(err)))
        }
    }
}

impl Service for WebService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<futures::Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        info!("{:?}", req);

        match (req.method(), self.router.test(req.path())) {
            // GET /healthcheck
            (&Get, Some(router::Route::Healthcheck)) => {
                let message = StatusMessage::new("OK");
                let response = serde_json::to_string(&message).unwrap();
                self.respond_with(Ok(response))
            },
            /*
            // POST /users
            (&Post, Some(router::Route::Users)) => {
                let conn = self.get_connection();

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
                                Ok(data) => future::ok(response_with_json(data)),
                                Err(err) => future::ok(response_with_error(error::Error::Json(err)))
                            }
                        })
                )
            },
            // PUT /users/1
            (&Put, Some(router::Route::User(user_id))) => {
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
                                Ok(data) => future::ok(response_with_json(data)),
                                Err(err) => future::ok(response_with_error(error::Error::Json(err)))
                            }
                        })
                )
            },
            */
            // GET /users/<user_id>
            (&Get, Some(router::Route::User(user_id))) => {
                let conn = self.get_connection();
                let result: Result<String, ApiError> = users.find(user_id).get_result::<User>(&*conn)
                    .map_err(|e| ApiError::from(e))
                    .and_then(|user| {
                        serde_json::to_string(&user)
                            .map_err(|e| ApiError::from(e))
                    });

                self.respond_with(result)
            },
            // GET /users
            (&Get, Some(router::Route::Users)) => {
                let conn = self.get_connection();
                let result: Result<String, ApiError> = req.uri().query()
                    .ok_or(ApiError::BadRequest("Missing query parameters: `from`, `count`".to_string()))
                    .and_then(|query| Ok(query_params(query)))
                    .and_then(|params| {
                        Ok((params.clone(), params.get("from").and_then(|v| v.parse::<i32>().ok())
                            .ok_or(ApiError::BadRequest("Invalid value provided for `from`".to_string()))))
                    })
                    .and_then(|(params, from)| {
                        Ok((from, params.get("count").and_then(|v| v.parse::<i64>().ok())
                            .ok_or(ApiError::BadRequest("Invalid value provided for `count`".to_string()))))
                    })
                    .and_then(|(from, count)| {
                        match (from, count) {
                            (Ok(x), Ok(y)) if x > 0 && y < MAX_USER_COUNT => Ok((x, y)),
                            (_, _) => Err(ApiError::BadRequest("Invalid values provided for `from` or `count`".to_string())),
                        }
                    })
                    .and_then(|(from, count)| {
                        let query = users.filter(is_active.eq(true)).filter(id.gt(from))
                            .order(id).limit(count);

                        query.load::<User>(&*conn)
                            .map_err(|e| ApiError::from(e))
                            .and_then(|results: Vec<User>| {
                                serde_json::to_string(&results)
                                    .map_err(|e| ApiError::from(e))
                            })
                    });

                self.respond_with(result)
            },
            // DELETE /users/<user_id>
            (&Delete, Some(router::Route::User(user_id))) => {
                let conn = self.get_connection();
                let query = users.filter(id.eq(user_id)).filter(is_active.eq(true));

                let result: Result<String, ApiError> = query.load::<User>(&*conn)
                    .map_err(|e| ApiError::from(e))
                    .and_then(|_user| {
                        diesel::update(query).set(is_active.eq(false)).get_result::<User>(&*conn)
                            .map_err(|e| ApiError::from(e))
                    })
                    .and_then(|_user| {
                        let message = StatusMessage::new("User has been deleted");
                        serde_json::to_string(&message).map_err(|e| ApiError::from(e))
                    });

                self.respond_with(result)
            },
            // Fallback
            _ => self.respond_with(Err(ApiError::NotFound))
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
