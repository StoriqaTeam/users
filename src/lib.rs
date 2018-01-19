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
extern crate hyper_tls;

#[macro_use]
pub mod macros;
pub mod app;
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
pub mod client;

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
use settings::Settings;

/// Starts new web service from provided `Settings`
pub fn start_server(settings: Settings) {
    // Prepare logger
    env_logger::init().unwrap();

    // Prepare reactor
    let mut core = Core::new().expect("Unexpected error creating event loop core");
    let handle = Arc::new(core.handle());

    let client = client::Client::new(&settings, &handle);
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
    let facebook_settings = settings.google.clone();


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

        // Prepare application
        let app = Application {
            router: Arc::new(router::create_router()),
            system_service: Arc::new(system_service),
            users_service: Arc::new(users_service),
            jwt_service: Arc::new(jwt_service)
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
