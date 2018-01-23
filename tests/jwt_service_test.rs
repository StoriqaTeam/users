extern crate hyper;
extern crate serde_json;
extern crate tokio_core;
extern crate users_lib;
extern crate futures;

use std::sync::Arc;
use std::time::SystemTime;

use tokio_core::reactor::Core;
use futures::Stream;

use users_lib::config::Config;
use users_lib::repos::users::UsersRepo;
use users_lib::repos::types::RepoFuture;
use users_lib::services::jwt::{JWTServiceImpl, JWTService};
use users_lib::models::user::{NewUser, UpdateUser, User, Gender, Provider};
use users_lib::models::jwt::ProviderOauth;
use users_lib::http::client::Client;


struct UsersRepoMock;

impl UsersRepo for UsersRepoMock {

    fn find(&self, user_id: i32) -> RepoFuture<User> {
        let user = create_user(user_id, MOCK_EMAIL.to_string());
        Box::new(futures::future::ok(user))
    }

    fn email_provider_exists(&self, email_arg: String, provider_arg: Provider) -> RepoFuture<bool> {
        Box::new(futures::future::ok(email_arg == MOCK_EMAIL.to_string()))
    }

    fn list(&self, from: i32, count: i64) -> RepoFuture<Vec<User>> {
        let mut users = vec![];
        for i in from..(from + count as i32) {
            let user = create_user(i, MOCK_EMAIL.to_string());
            users.push(user);
        }
        Box::new(futures::future::ok(users))
    }

    fn create(&self, payload: NewUser) -> RepoFuture<User> {
        let user = create_user(1, payload.user_email);
        Box::new(futures::future::ok(user))
    }

    fn update(&self, user_id: i32, payload: UpdateUser) -> RepoFuture<User> {
        let user = create_user(user_id, payload.email);
        Box::new(futures::future::ok(user))
    }

    fn deactivate(&self, user_id: i32) -> RepoFuture<User> {
        let mut user = create_user(user_id, MOCK_EMAIL.to_string());
        user.is_active = false;
        Box::new(futures::future::ok(user))
    }

    fn verify_password(&self, email_arg: String, password_arg: String) -> RepoFuture<bool> {
        Box::new(futures::future::ok(email_arg == MOCK_EMAIL.to_string() && password_arg == MOCK_PASSWORD.to_string()))
    }
}

fn create_service () -> (Core, JWTServiceImpl<UsersRepoMock>) {
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
    let service = JWTServiceImpl { 
            users_repo : Arc::new(MOCK), 
            http_client: client_handle,
            google_settings: google_settings,
            facebook_settings: facebook_settings,
            jwt_settings: jwt_settings,
    };
    (core, service)
}


fn create_user(id: i32, email: String) -> User {
    User {
        id: id,
        email: email,
        email_verified: false,
        phone: None,
        phone_verified: false,
        is_active: true,
        first_name: None,
        last_name: None,
        middle_name: None,
        gender: Gender::Male,
        birthdate: None,
        last_login_at: SystemTime::now(), 
        created_at: SystemTime::now(), 
        updated_at: SystemTime::now()
    }
}

fn create_new_user(email: String, password: String) -> NewUser {
    NewUser {
        user_email: email,
        user_password: password,
    }
}


const MOCK : UsersRepoMock = UsersRepoMock{};
static MOCK_EMAIL: &'static str = "example@mail.com";
static MOCK_PASSWORD: &'static str = "password";
static GOOGLE_CODE: &'static str = "google";
static FACEBOOK_CODE: &'static str = "AQDr-FG4bmYyrhYGk9ZJg1liqTRBfKfRbXopSd72_Qjexg3e4ybh9EJZFErHwyhw0oKyUOEbCQSalC4D8b3B2r4eJiyEmyW-E_ESsVnyThn27j8KEDDfsxCwUJxZY6fDwZt9LWMEHnHYEnFxABIupKN8y8bj_SH8wxIZoDm-YzZtYbj7VUf9g0vPKOkA_1hnjjW8TGrEKmbhFZLWLj6wJgC3uek3D3MahUhd_k3K-4BjOJNyXa8h_ESPQWNHt9sIIIDmhAw5X4iVmdbte7tQWf6y96vd_muwA4hKMRxzc7gMQo16tcI7hazQaJ1rJj39G8poG9Ac7AjdO6O7vSnYB9IqeLFbhKH56IyJoCR_05e2tg";

#[test]
fn test_jwt_email() {
    let (mut core, service) = create_service();
    let new_user = create_new_user (MOCK_EMAIL.to_string(), MOCK_PASSWORD.to_string());

    let work = service.create_token_email(new_user);
    let result = core.run(work).unwrap();
    assert_eq!(result.token, "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2VtYWlsIjoiZXhhbXBsZUBtYWlsLmNvbSJ9.EiRpbadz8jGW0_wGPKXKhlmrWC9QJNIDv8eRWp0-VG0");
}

#[test]
fn test_jwt_email_not_found() {
    let (mut core, service) = create_service();
    let new_user = create_new_user ("not found email".to_string(), MOCK_PASSWORD.to_string());
    let work = service.create_token_email(new_user);
    let result = core.run(work);
    assert_eq!(result.is_err(), true);
}

#[test]
fn test_jwt_password_incorrect() {
    let (mut core, service) = create_service();
    let new_user = create_new_user (MOCK_EMAIL.to_string(), "wrong password".to_string());
    let work = service.create_token_email(new_user);
    let result = core.run(work);
    assert_eq!(result.is_err(), true);
}

// this test is ignored because of expired access code from google 
#[test]
#[ignore] 
fn test_jwt_google() {
    let (mut core, service) = create_service();
    let oauth = ProviderOauth { code: GOOGLE_CODE.to_string() };
    let work = service.create_token_google(oauth);
    let result = core.run(work).unwrap();
    assert_eq!(result.token, "token");
}

// this test is ignored because of expired access code from google 
#[test]
#[ignore]
fn test_jwt_facebook() {
    let (mut core, service) = create_service();
    let oauth = ProviderOauth { code: FACEBOOK_CODE.to_string() };
    let work = service.create_token_facebook(oauth);
    let result = core.run(work).unwrap();
    assert_eq!(result.token, "token");
}