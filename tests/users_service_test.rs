extern crate hyper;
extern crate serde_json;
extern crate tokio_core;
extern crate users_lib;
extern crate futures;
#[macro_use]
extern crate lazy_static;

use std::sync::Arc;

use tokio_core::reactor::Core;

use users_lib::models::user::User;
use users_lib::repos::users::UsersRepo;
use users_lib::repos::types::RepoFuture;
use users_lib::services::users::{UsersServiceImpl, UsersService};
use users_lib::payloads::user::{NewUser, UpdateUser};

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

const MOCK : UsersRepoMock = UsersRepoMock{};
lazy_static! {
    static ref SERVICE : UsersServiceImpl<UsersRepoMock> = UsersServiceImpl { users_repo : Arc::new(MOCK) };
    static ref MOCK_EMAIL: String = "example@mail.com".to_string();
    static ref MOCK_PASSWORD: String = "password".to_string();
}


#[test]
fn test_get_user() {
    let mut core = Core::new().unwrap();
    let work = SERVICE.get(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
}

#[test]
fn test_list() {
    let mut core = Core::new().unwrap();
    let work = SERVICE.list(1, 5);
    let result = core.run(work).unwrap();
    assert_eq!(result.len(), 5);
}

#[test]
#[should_panic]
fn test_create_allready_existed() {
    let mut core = Core::new().unwrap();
    let new_user = NewUser { email: MOCK_EMAIL.to_string(), password: MOCK_PASSWORD.to_string() };
    let work = SERVICE.create(new_user);
    core.run(work).unwrap();
}

#[test]
fn test_create_user() {
    let mut core = Core::new().unwrap();
    let new_user = NewUser { email: "new_user@mail.com".to_string(), password: MOCK_PASSWORD.to_string() };
    let work = SERVICE.create(new_user);
    let result = core.run(work).unwrap();
    assert_eq!(result.email, "new_user@mail.com".to_string());
}

#[test]
fn test_update() {
    let mut core = Core::new().unwrap();
    let update_user = UpdateUser {email: MOCK_EMAIL.to_string()};
    let work = SERVICE.update(1, update_user);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.email, MOCK_EMAIL.to_string());
}

#[test]
fn test_deactivate() {
    let mut core = Core::new().unwrap();
    let work = SERVICE.deactivate(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.is_active, false);
}