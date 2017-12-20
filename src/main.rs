extern crate users_lib;

fn main() {
    let settings = users_lib::settings::Settings::new().expect("Can't load users settings!");
    users_lib::start_server(settings);
}
