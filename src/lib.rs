extern crate config;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;
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
pub mod facades;
pub mod router;
pub mod models;
pub mod payloads;
pub mod repos;
pub mod responses;
pub mod services;
pub mod settings;
pub mod utils;

use std::sync::Arc;
use std::process;

use futures::{Future, Stream};
use futures::future;
use futures_cpupool::CpuPool;
use hyper::{Get, Post, Put, Delete};
use hyper::server::{Http, Service, Request};
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;
use tokio_core::reactor::Core;

use common::{TheError, TheFuture, TheRequest, TheResponse};
use error::Error as ApiError;
use facades::system::SystemFacade;
use facades::users::UsersFacade;
use repos::users::UsersRepo;
use router::{Route, Router};
use services::system::SystemService;
use services::users::UsersService;
use settings::Settings;
use utils::http::response_with_error;

/// WebService containing all sub-crate services and `Router`
struct WebService {
    router: Arc<Router>,
    system_facade: Arc<SystemFacade>,
    users_facade: Arc<UsersFacade>,
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
            (&Get, Some(Route::Healthcheck)) => self.system_facade.healthcheck(),
            // GET /users/<user_id>
            (&Get, Some(Route::User(user_id))) => self.users_facade.get(user_id),
            // GET /users
            (&Get, Some(Route::Users)) => self.users_facade.list(req),
            // POST /users
            (&Post, Some(Route::Users)) => self.users_facade.create(req),
            // PUT /users/<user_id>
            (&Put, Some(Route::User(user_id))) => self.users_facade.update(req, user_id),
            // DELETE /users/<user_id>
            (&Delete, Some(Route::User(user_id))) => self.users_facade.deactivate(user_id),
            // Fallback
            _ => Box::new(future::ok(response_with_error(ApiError::NotFound)))
        }
    }
}

/// Starts new web service from provided `Settings`
pub fn start_server(settings: Settings) {
    // Prepare logger
    env_logger::init().unwrap();

    // Prepare reactor
    let mut core = Core::new().expect("Unexpected error creating event loop core");
    let handle = Arc::new(core.handle());

    // Prepare server
    let address = settings.address.parse().expect("Address must be set in configuration");
    let serve = Http::new().serve_addr_handle(&address, &handle, move || {
        // Prepare database pool
        let database_url: String = settings.database.parse().expect("Database URL must be set in configuration");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let r2d2_pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create connection pool");

        let cpu_pool = CpuPool::new(10);

        // Prepare repositories
        let users_repo = UsersRepo {
            r2d2_pool: Arc::new(r2d2_pool),
            cpu_pool: Arc::new(cpu_pool),
        };

        // Prepare services
        let system_service = SystemService{};

        let users_service = UsersService {
            users_repo: Arc::new(users_repo)
        };

        // Prepare facades
        let system_facade = SystemFacade {
            system_service: Arc::new(system_service)
        };

        let users_facade = UsersFacade {
            users_service: Arc::new(users_service)
        };

        // Prepare final service
        let service = WebService {
            router: Arc::new(router::create_router()),
            system_facade: Arc::new(system_facade),
            users_facade: Arc::new(users_facade)
        };

        Ok(service)
    }).unwrap_or_else(|why| {
        error!("Http Server Initialization Error: {}", why);
        process::exit(1);
    });

    let handle_arc2 = handle.clone();
    handle.spawn(
        serve.for_each(move |conn| {
            handle_arc2.spawn(
                conn.map(|_| ())
                    .map_err(|why| error!("Server Error: {:?}", why)),
            );
            Ok(())
        })
        .map_err(|_| ()),
    );

    info!("Listening on http://{}", address);
    core.run(future::empty::<(), ()>()).unwrap();
}
