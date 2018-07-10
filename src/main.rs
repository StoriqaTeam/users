//! Users is a microservice responsible for authentication and managing user profiles.
//! This create is for running the service from `users_lib`. See `users_lib` for details.

extern crate users_lib;
extern crate stq_logging;

fn main() {
    let config = users_lib::config::Config::new().expect("Can't load app config!");

    // Prepare logger
    stq_logging::init(config.graylog.as_ref());

    users_lib::start_server(config);
}
