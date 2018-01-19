extern crate users_lib;

fn main() {
    let config = users_lib::config::Config::new().expect("Can't load app config!");
    users_lib::start_server(config);
}
