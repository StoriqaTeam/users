extern crate hyper;
extern crate serde_json;
extern crate tokio_core;
extern crate users_lib;
extern crate futures;

use std::sync::Arc;

use tokio_core::reactor::Core;
use futures::Stream;
use hyper::{Request, Method};
use hyper::header::Authorization;


use users_lib::config::Config;
use users_lib::models::user::{User, NewUser, UpdateUser};
use users_lib::services::jwt::JWTServiceImpl;
use users_lib::services::users::UsersServiceImpl;
use users_lib::services::system::SystemServiceImpl;
use users_lib::controller::Controller;
use users_lib::repos::users::UsersRepo;
use users_lib::repos::types::RepoFuture;
use users_lib::http::client::Client;


struct UsersRepoMock;

impl UsersRepo for UsersRepoMock {

    fn find(&self, user_id: i32) -> RepoFuture<User> {
        let user = User {  id: user_id, email: MOCK_EMAIL.to_string(), password: MOCK_PASSWORD.to_string(), is_active: true };
        Box::new(futures::future::ok(user))
    }

    fn find_by_email(&self, email_arg: String) -> RepoFuture<User>{
        let user = User {  id: 1, email: email_arg.to_string(), password: MOCK_PASSWORD.to_string(), is_active: true };
        Box::new(futures::future::ok(user))
    }

    fn email_exists(&self, email_arg: String) -> RepoFuture<bool> {
        Box::new(futures::future::ok(email_arg == MOCK_EMAIL.to_string()))
    }

    fn list(&self, from: i32, count: i64) -> RepoFuture<Vec<User>> {
        let mut users = vec![];
        for i in from..(from + count as i32) {
            let user = User {  id: i, email: MOCK_EMAIL.to_string(), password: MOCK_PASSWORD.to_string(), is_active: true };
            users.push(user);
        }
        Box::new(futures::future::ok(users))
    }

    fn create(&self, payload: NewUser) -> RepoFuture<User> {
        let user = User {  id: 1, email: payload.email, password: payload.password, is_active: true };
        Box::new(futures::future::ok(user))
    }

    fn update(&self, user_id: i32, payload: UpdateUser) -> RepoFuture<User> {
        let user = User {  id: user_id, email: payload.email, password: MOCK_PASSWORD.to_string(), is_active: true };
        Box::new(futures::future::ok(user))
    }

    fn deactivate(&self, user_id: i32) -> RepoFuture<User> {
        let user = User {  id: user_id, email: MOCK_EMAIL.to_string(), password: MOCK_PASSWORD.to_string(), is_active: false };
        Box::new(futures::future::ok(user))
    }

    fn verify_password(&self, email_arg: String, password_arg: String) -> RepoFuture<bool> {
        Box::new(futures::future::ok(email_arg == MOCK_EMAIL.to_string() && password_arg == MOCK_PASSWORD.to_string()))
    }
}

fn create_controller() -> (Core, Controller) {
    let config = Config::new().unwrap();
    let core = Core::new().expect("Unexpected error creating event loop core");
    let handle = Arc::new(core.handle());
    let client = Client::new(&config, &handle);
    let client_handle = client.handle();
    let client_stream = client.stream();
    handle.spawn(
        client_stream.for_each(|_| Ok(()))
    );
    let jwt_settings = config.jwt.clone();
    let google_settings = config.google.clone();
    let facebook_settings = config.facebook.clone();

    let jwt_service = Arc::new(JWTServiceImpl { 
            users_repo : Arc::new(MOCK), 
            http_client: client_handle,
            google_settings: google_settings,
            facebook_settings: facebook_settings,
            jwt_settings: jwt_settings,
    });
    let users_service = Arc::new(UsersServiceImpl { users_repo : Arc::new(MOCK) });
    let system_service = Arc::new(SystemServiceImpl{});
        
    let controller = Controller::new(system_service, users_service, jwt_service);

    (core, controller)
}

const MOCK : UsersRepoMock = UsersRepoMock{};
static MOCK_EMAIL: &'static str = "example@mail.com";
static MOCK_PASSWORD: &'static str = "password";


#[test]
fn test_user_id() {
    let (mut core, controller) = create_controller();
    let uri = "/users/1".parse().unwrap();
    let req = Request::new(Method::Get, uri);

    let work = controller.call(req);
    let result = core.run(work).unwrap();

    let user = User {  id: 1, email: MOCK_EMAIL.to_string(), password: MOCK_PASSWORD.to_string(), is_active: true };
    let expected =  serde_json::to_string(&user).unwrap(); 

    assert_eq!(result, expected);
}

#[test]
fn test_user_current() {
    let (mut core, controller) = create_controller();
    let uri = "/users/current".parse().unwrap();
    let mut req = Request::new(Method::Get, uri);
    req.headers_mut().set(Authorization(MOCK_EMAIL.to_string()));

    let work = controller.call(req);
    let result = core.run(work).unwrap();

    let user = User {  id: 1, email: MOCK_EMAIL.to_string(), password: MOCK_PASSWORD.to_string(), is_active: true };
    let expected =  serde_json::to_string(&user).unwrap(); 

    assert_eq!(result, expected);
}

#[test]
fn test_user_current_no_auth() {
    let (mut core, controller) = create_controller();
    let uri = "/users/current".parse().unwrap();
    let req = Request::new(Method::Get, uri);

    let work = controller.call(req);
    let result = core.run(work);

    assert_eq!(result.is_err(), true);
}
