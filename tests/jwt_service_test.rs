extern crate hyper;
extern crate serde_json;
extern crate tokio_core;
extern crate users_lib;
extern crate futures;

use std::sync::Arc;

use tokio_core::reactor::Core;
use futures::Stream;

use users_lib::models::user::User;
use users_lib::config::Config;
use users_lib::repos::users::UsersRepo;
use users_lib::repos::types::RepoFuture;
use users_lib::services::jwt::{JWTServiceImpl, JWTService};
use users_lib::models::user::{NewUser, UpdateUser};
use users_lib::models::jwt::ProviderOauth;
use users_lib::http::client::Client;


struct UsersRepoMock;

impl UsersRepo for UsersRepoMock {

    fn find(&self, user_id: i32) -> RepoFuture<User> {
        let user = User {  id: user_id, email: MOCK_EMAIL.to_string(), password: MOCK_PASSWORD.to_string(), is_active: true };
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
    let facebook_settings = config.google.clone();
    let service = JWTServiceImpl { 
            users_repo : Arc::new(MOCK), 
            http_client: client_handle,
            google_settings: google_settings,
            facebook_settings: facebook_settings,
            jwt_settings: jwt_settings,
    };
    (core, service)
}

const MOCK : UsersRepoMock = UsersRepoMock{};
static MOCK_EMAIL: &'static str = "example@mail.com";
static MOCK_PASSWORD: &'static str = "password";
static GOOGLE_CODE: &'static str = "google";
static FACEBOOK_CODE: &'static str = "AQC3iQv4gqqaEHdKTL-k8RbQKMa940pbEEyfDZeddNKLnWJLI1J8nhsylbUNqkIhT6efI2fQhqGylYw5pQP8ECtdYcrjVX4rTOEIag_WsH6KRf37WBN1iRZQr_QQJKAamK-LG691LsVJJXA66YbLmfx2lTqBrRs6qRJ62fEk_fOwUsGy8M4GbA3L2foyFQp5cZYb_8l11Ada8C2sAlHWEGQpBb0Mm7I3aCZNUSqanZrZbkEajMxHe_0Eei27eTu9rDcBjUnPAmj-LkQBiyu8PT-ActeY9x1iiPN5pLdijbF6_x_UPj7yWSEXog0MQC2JzZ5NzaoqdnC8vgPqFOqtpEj3kr9vhDIPcbXPKZLJ8cQuaQ";

#[test]
fn test_jwt_email() {
    let (mut core, service) = create_service();
    let new_user = NewUser { email: MOCK_EMAIL.to_string(), password: MOCK_PASSWORD.to_string() };
    let work = service.create_token_email(new_user);
    let result = core.run(work).unwrap();
    assert_eq!(result.token, "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2VtYWlsIjoiZXhhbXBsZUBtYWlsLmNvbSJ9.EiRpbadz8jGW0_wGPKXKhlmrWC9QJNIDv8eRWp0-VG0");
}

#[test]
fn test_jwt_google() {
    let (mut core, service) = create_service();
    let oauth = ProviderOauth { code: GOOGLE_CODE.to_string() };
    let work = service.create_token_google(oauth);
    let result = core.run(work).unwrap();
    assert_eq!(result.token, "token");
}

#[test]
fn test_jwt_facebook() {
    let (mut core, service) = create_service();
    let oauth = ProviderOauth { code: FACEBOOK_CODE.to_string() };
    let work = service.create_token_facebook(oauth);
    let result = core.run(work).unwrap();
    assert_eq!(result.token, "token");
}