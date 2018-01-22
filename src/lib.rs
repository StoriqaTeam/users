//! Users is a microservice responsible for authentication and managing user profiles.
//! The layered structure of the app is
//!
//! `Application -> Controller -> Service -> Repo + HttpClient`
//!
//! Each layer can only face exceptions in its base layers and can only expose its own errors.
//! E.g. `Service` layer will only deal with `Repo` and `HttpClient` errors and will only return
//! `ServiceError`. That way Controller will only have to deal with ServiceError, but not with `Repo`
//! or `HttpClient` repo.

extern crate config as config_crate;
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
extern crate hyper_tls;

#[macro_use]
pub mod macros;
pub mod app;
pub mod authorization;
pub mod controller;
pub mod models;
pub mod repos;
pub mod services;
pub mod config;
pub mod types;
pub mod http;

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
use repos::users::UsersRepoImpl;
use services::system::SystemService;
use services::users::UsersServiceImpl;
use services::jwt::JWTServiceImpl;
use config::Config;

/// Starts new web service from provided `Config`
pub fn start_server(settings: Config) {
    // Prepare logger
    env_logger::init().unwrap();

    // Prepare reactor
    let mut core = Core::new().expect("Unexpected error creating event loop core");
    let handle = Arc::new(core.handle());

    let client = http::client::Client::new(&settings, &handle);
    let client_handle = client.handle();
    let client_stream = client.stream();
    handle.spawn(
        client_stream.for_each(|_| Ok(()))
    );

    // Prepare server
    let thread_count = settings.server.thread_count.clone();
    let address = settings.server.address.parse().expect("Address must be set in configuration");
    let jwt_settings = settings.jwt.clone();
    let google_settings = settings.google.clone();
    let facebook_settings = settings.facebook.clone();


    let serve = Http::new().serve_addr_handle(&address, &handle, move || {
        // Prepare database pool
        let database_url: String = settings.server.database.parse().expect("Database URL must be set in configuration");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let r2d2_pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create connection pool");

        // Prepare CPU pool
        let cpu_pool = CpuPool::new(thread_count);

        // Prepare repositories
        let users_repo = UsersRepoImpl {
            r2d2_pool: Arc::new(r2d2_pool),
            cpu_pool: Arc::new(cpu_pool),
        };

         // Prepare services
        let system_service = SystemService{};

        let users_repo = Arc::new(users_repo);

        let users_service = UsersServiceImpl {
            users_repo: users_repo.clone(),
        };

        let jwt_service = JWTServiceImpl {
            users_repo: users_repo.clone(),
            http_client: client_handle.clone(),
            jwt_settings: jwt_settings.clone(),
            google_settings: google_settings.clone(),
            facebook_settings: facebook_settings.clone(),

        };

        let controller = controller::Controller::new(Arc::new(system_service), Arc::new(users_service), Arc::new(jwt_service));

        // Prepare application
        let app = Application {
            controller,
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

    info!("Listening on http://{}, threads: {}", address, thread_count);
    core.run(future::empty::<(), ()>()).unwrap();
}
