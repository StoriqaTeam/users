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
pub mod models;
pub mod payloads;
pub mod repos;
pub mod responses;
pub mod services;
pub mod settings;
pub mod utils;

use std::sync::Arc;

use futures::Future;
use futures::future;
use hyper::{Get, Post, Put, Delete};
use hyper::server::{Http, Service, Request};
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;

use common::{TheError, TheFuture, TheRequest, TheResponse};
use error::Error as ApiError;
use repos::users::UsersRepo;
use router::Route;
use services::system::SystemService;
use services::users::UsersService;
use settings::Settings;
use utils::http::response_with_error;

/// WebService containing all sub-crate services and `Router`
struct WebService {
    router: Arc<router::Router>,
    system_service: Arc<SystemService>,
    users_service: Arc<UsersService>,
}

impl Service for WebService {
    type Request = TheRequest;
    type Response = TheResponse;
    type Error = TheError;
    type Future = TheFuture;

    fn call(&self, req: Request) -> Box<Future<Item = TheResponse, Error = TheError>> {
        info!("{:?}", req);

        match (req.method(), self.router.test(req.path())) {
            // GET /healthcheck
            (&Get, Some(Route::Healthcheck)) => self.system_service.healthcheck(),
            // GET /users/<user_id>
            (&Get, Some(Route::User(user_id))) => self.users_service.get(user_id),
            // GET /users
            (&Get, Some(Route::Users)) => self.users_service.list(req),
            // POST /users
            (&Post, Some(Route::Users)) => self.users_service.create(req),
            // PUT /users/<user_id>
            (&Put, Some(Route::User(user_id))) => self.users_service.update(req, user_id),
            // DELETE /users/<user_id>
            (&Delete, Some(Route::User(user_id))) => self.users_service.deactivate(user_id),
            // Fallback
            _ => Box::new(future::ok(response_with_error(ApiError::NotFound)))
        }
    }
}

/// Starts new web service from provided `Settings`
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
