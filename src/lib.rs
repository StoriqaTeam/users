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

pub mod common;
pub mod error;
pub mod router;
pub mod http_utils;
pub mod schema;
pub mod models;
pub mod payloads;
pub mod service;
pub mod settings;
pub mod system_service;
pub mod users_repo;
pub mod users_service;

use std::sync::Arc;

use futures::future;
//use futures::Future;
use hyper::{Get, Post, Put, Delete};
use hyper::server::{Http, Service, Request};
//use diesel::prelude::*;
use diesel::pg::PgConnection;
//use diesel::select;
//use diesel::dsl::exists;
use r2d2_diesel::ConnectionManager;
//use validator::Validate;

use common::{TheError, TheFuture, TheRequest, TheResponse};
use error::Error as ApiError;
//use error::StatusMessage;
use http_utils::response_with_error;
//use models::*;
//use schema::users::dsl::*;
use settings::Settings;
//use payloads::{NewUser, UpdateUser};
use system_service::SystemService;
use users_repo::UsersRepo;
use users_service::UsersService;

struct WebService {
    router: Arc<router::Router>,
    system_service: Arc<SystemService>,
    users_service: Arc<UsersService>,
}

//impl WebService {
//    fn get_connection(&self) -> TheConnection {
//        match self.pool.get() {
//            Ok(connection) => connection,
//            Err(e) => panic!("Error obtaining connection from pool: {}", e)
//        }
//    }
//
//    fn respond_with(&self, result: Result<String, ApiError>) -> <WebService as Service>::Future {
//        match result {
//            Ok(response) => Box::new(future::ok(response_with_json(response))),
//            Err(err) => Box::new(future::ok(response_with_error(err)))
//        }
//    }
//}

impl Service for WebService {
    type Request = TheRequest;
    type Response = TheResponse;
    type Error = TheError;
    type Future = TheFuture;

    fn call(&self, req: Request) -> Self::Future {
        info!("{:?}", req);

        match (req.method(), self.router.test(req.path())) {
            // GET /healthcheck
            (&Get, Some(router::Route::Healthcheck)) => self.system_service.healthcheck(),
            // GET /users/<user_id>
            (&Get, Some(router::Route::User(user_id))) => self.users_service.find(user_id),
            // GET /users
            (&Get, Some(router::Route::Users)) => self.users_service.list(req),
            /*
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
            */
            // PUT /users/1
            (&Put, Some(router::Route::User(user_id))) => self.users_service.update(req, user_id),
            // DELETE /users/<user_id>
            (&Delete, Some(router::Route::User(user_id))) => self.users_service.deactivate(user_id),
            // Fallback
            _ => Box::new(future::ok(response_with_error(ApiError::NotFound)))
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

        // Prepare repositories
        let users_repo = UsersRepo {
            r2d2_pool: Arc::new(r2d2_pool),
        };

        // Prepare services
        let system_service = SystemService{};

        let users_service = UsersService {
            users_repo: Arc::new(users_repo)
        };

        // Prepare final service
        let service = WebService {
            router: Arc::new(router::create_router()),
            system_service: Arc::new(system_service),
            users_service: Arc::new(users_service),
        };

        Ok(service)
    }).unwrap();

    server.no_proto();

    // Start
    info!("Listening on http://{}", server.local_addr().unwrap());
    server.run().unwrap();
}
