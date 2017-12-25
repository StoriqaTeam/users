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
#[macro_use]
extern crate validator_derive;
extern crate validator;

pub mod error;
pub mod router;
pub mod http_utils;
pub mod schema;
pub mod models;
pub mod payloads;
pub mod settings;

use std::sync::Arc;

use futures::future;
use futures::Future;
use hyper::{Get, Post, Put, Delete};
use hyper::server::{Http, Service, Request, Response};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::select;
use diesel::dsl::exists;
use r2d2_diesel::ConnectionManager;
use validator::Validate;

use error::Error as ApiError;
use error::StatusMessage;
use http_utils::*;
use models::*;
use schema::users::dsl::*;
use settings::Settings;
use payloads::{NewUser, UpdateUser};

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
                        // Extract `from` param
                        Ok((params.clone(), params.get("from").and_then(|v| v.parse::<i32>().ok())
                            .ok_or(ApiError::BadRequest("Invalid value provided for `from`".to_string()))))
                    })
                    .and_then(|(params, from)| {
                        // Extract `count` param
                        Ok((from, params.get("count").and_then(|v| v.parse::<i64>().ok())
                            .ok_or(ApiError::BadRequest("Invalid value provided for `count`".to_string()))))
                    })
                    .and_then(|(from, count)| {
                        // Transform tuple of `Result`s to `Result` of tuple
                        match (from, count) {
                            (Ok(x), Ok(y)) if x > 0 && y < MAX_USER_COUNT => Ok((x, y)),
                            (_, _) => Err(ApiError::BadRequest("Invalid values provided for `from` or `count`".to_string())),
                        }
                    })
                    .and_then(|(from, count)| {
                        // Get users
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
            // POST /users
            (&Post, Some(router::Route::Users)) => {
                let conn = self.get_connection();

                Box::new(
                    read_body(req)
                        .and_then(move |body| {
                            let result: Result<String, ApiError> = serde_json::from_slice::<NewUser>(&body.as_bytes())
                                .map_err(|e| ApiError::from(e))
                                .and_then(|new_user| {
                                    // General validation
                                    match new_user.validate() {
                                        Ok(_) => Ok(new_user),
                                        Err(e) => Err(ApiError::from(e))
                                    }
                                })
                                .and_then(|new_user| {
                                    // Unique e-mail validation
                                    let count = select(exists(users.filter(email.eq(new_user.email)))).get_result(&*conn);
                                    match count {
                                        Ok(false) => Ok(new_user),
                                        Ok(true) => Err(ApiError::BadRequest("E-mail already registered".to_string())),
                                        Err(e) => Err(ApiError::from(e))
                                    }
                                })
                                .and_then(|new_user| {
                                    // User creation
                                    let query = diesel::insert_into(users).values(&new_user);

                                    query.get_result::<User>(&*conn)
                                        .map_err(|e| ApiError::from(e))
                                        .and_then(|user: User| {
                                            serde_json::to_string(&user)
                                                .map_err(|e| ApiError::from(e))
                                        })
                                });

                            match result {
                                Ok(data) => future::ok(response_with_json(data)),
                                Err(err) => future::ok(response_with_error(ApiError::from(err)))
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
                            let result: Result<String, ApiError> = users.find(user_id).get_result::<User>(&*conn)
                                .map_err(|e| ApiError::from(e))
                                .and_then(|_user| {
                                    serde_json::from_slice::<UpdateUser>(&body.as_bytes())
                                        .map_err(|e| ApiError::from(e))
                                })
                                .and_then(|new_user| {
                                    // TODO: Update other fields, don't update e-mail at all
                                    let filter = users.filter(id.eq(user_id)).filter(is_active.eq(true));
                                    let query = diesel::update(filter).set(email.eq(new_user.email));

                                    query.get_result::<User>(&*conn)
                                        .map_err(|e| ApiError::from(e))
                                        .and_then(|user: User| {
                                            serde_json::to_string(&user)
                                                .map_err(|e| ApiError::from(e))
                                        })
                                });

                            match result {
                                Ok(data) => future::ok(response_with_json(data)),
                                Err(err) => future::ok(response_with_error(ApiError::from(err)))
                            }
                        })
                )
            },
            // DELETE /users/<user_id>
            (&Delete, Some(router::Route::User(user_id))) => {
                let conn = self.get_connection();
                let query = users.filter(id.eq(user_id)).filter(is_active.eq(true));

                let result: Result<String, ApiError> = query.load::<User>(&*conn)
                    .map_err(|e| ApiError::from(e))
                    .and_then(|_user| {
                        let query = diesel::update(query).set(is_active.eq(false));

                        query.get_result::<User>(&*conn)
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
