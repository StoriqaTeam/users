extern crate hyper;
extern crate serde_json;
extern crate tokio_core;
extern crate users_lib;
extern crate futures;

use std::sync::Arc;

use tokio_core::reactor::Core;

use users_lib::repos::users::UsersRepo;
use users_lib::repos::types::RepoFuture;
use users_lib::services::users::{UsersServiceImpl, UsersService};
use users_lib::models::user::{NewUser, UpdateUser, User};
use users_lib::services::context::Context;

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

    fn verify_password(&self, _email_arg: String, _password_arg: String) -> RepoFuture<bool> {
        Box::new(futures::future::ok(true))
    }
}

fn create_service (context: Context) -> UsersServiceImpl<UsersRepoMock> {
    UsersServiceImpl::new( Arc::new(MOCK), context ) 
}

const MOCK : UsersRepoMock = UsersRepoMock{};
static MOCK_EMAIL: &'static str = "example@mail.com";
static MOCK_PASSWORD: &'static str = "password";
const CONTEXT_WITHOUT_EMAIL : Context = Context { user_email: None };


#[test]
fn test_get_user() {
    let context = Context { user_email: Some(MOCK_EMAIL.to_string()) };
    let service = create_service(context);
    let mut core = Core::new().unwrap();
    let work = service.get(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
}

#[test]
fn test_current_user() {
    let context = Context { user_email: Some(MOCK_EMAIL.to_string()) };
    let service = create_service(context);
    let mut core = Core::new().unwrap();
    let work = service.current();
    let result = core.run(work).unwrap();
    assert_eq!(result.email, MOCK_EMAIL.to_string());
}

#[test]
fn test_current_user_without_user_email() {
    let context = CONTEXT_WITHOUT_EMAIL;
    let service = create_service(context);
    let mut core = Core::new().unwrap();
    let work = service.current();
    let result = core.run(work);
    assert_eq!(result.is_err(), true);
}

#[test]
fn test_list() {
    let context = Context { user_email: Some(MOCK_EMAIL.to_string()) };
    let service = create_service(context);
    let mut core = Core::new().unwrap();
    let work = service.list(1, 5);
    let result = core.run(work).unwrap();
    assert_eq!(result.len(), 5);
}

#[test]
fn test_create_allready_existed() {
    let context = Context { user_email: Some(MOCK_EMAIL.to_string()) };
    let service = create_service(context);
    let mut core = Core::new().unwrap();
    let new_user = NewUser { email: MOCK_EMAIL.to_string(), password: MOCK_PASSWORD.to_string() };
    let work = service.create(new_user);
    let result = core.run(work);
    assert_eq!(result.is_err(), true);
}

#[test]
fn test_create_user() {
    let context = Context { user_email: Some(MOCK_EMAIL.to_string()) };
    let service = create_service(context);
    let mut core = Core::new().unwrap();
    let new_user = NewUser { email: "new_user@mail.com".to_string(), password: MOCK_PASSWORD.to_string() };
    let work = service.create(new_user);
    let result = core.run(work).unwrap();
    assert_eq!(result.email, "new_user@mail.com".to_string());
}

#[test]
fn test_update() {
    let context = Context { user_email: Some(MOCK_EMAIL.to_string()) };
    let service = create_service(context);
    let mut core = Core::new().unwrap();
    let update_user = UpdateUser {email: MOCK_EMAIL.to_string()};
    let work = service.update(1, update_user);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.email, MOCK_EMAIL.to_string());
}

#[test]
fn test_deactivate() {
    let context = Context { user_email: Some(MOCK_EMAIL.to_string()) };
    let service = create_service(context);
    let mut core = Core::new().unwrap();
    let work = service.deactivate(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.is_active, false);
}