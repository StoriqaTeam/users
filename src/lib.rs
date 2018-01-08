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
extern crate jsonwebtoken;

pub mod app;
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
use hyper::server::Http;
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;
use tokio_core::reactor::Core;

use app::Application;
use facades::system::SystemFacade;
use facades::users::UsersFacade;
use repos::users::UsersRepo;
use repos::jwt::JWTRepo;
use services::system::SystemService;
use services::users::UsersService;
use settings::Settings;

/// Starts new web service from provided `Settings`
pub fn start_server(settings: Settings) {
    // Prepare logger
    env_logger::init().unwrap();

    // Prepare reactor
    let mut core = Core::new().expect("Unexpected error creating event loop core");
    let handle = Arc::new(core.handle());

    // Prepare server
    let threads = settings.threads.clone();
    let address = settings.address.parse().expect("Address must be set in configuration");
    let secret_key = settings.secret_key.clone();

    let serve = Http::new().serve_addr_handle(&address, &handle, move || {
        // Prepare database pool
        let database_url: String = settings.database.parse().expect("Database URL must be set in configuration");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let r2d2_pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create connection pool");

        // Prepare CPU pool
        let cpu_pool = CpuPool::new(settings.threads);

        // Prepare repositories
        let users_repo = UsersRepo {
            r2d2_pool: Arc::new(r2d2_pool),
            cpu_pool: Arc::new(cpu_pool),
        };

        let jwt_repo = JWTRepo{
            secret_key: secret_key
        }

        // Prepare services
        let system_service = SystemService{};

        let users_service = UsersService {
            users_repo: Arc::new(users_repo),
            jwt_repo: Arc::new(jwt_repo)
        };

        // Prepare facades
        let system_facade = SystemFacade {
            system_service: Arc::new(system_service)
        };

        let users_facade = UsersFacade {
            users_service: Arc::new(users_service)
        };

        // Prepare application
        let app = Application {
            router: Arc::new(router::create_router()),
            system_facade: Arc::new(system_facade),
            users_facade: Arc::new(users_facade)
        };

        Ok(app)
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

    info!("Listening on http://{}, threads: {}", address, threads);
    core.run(future::empty::<(), ()>()).unwrap();
}
