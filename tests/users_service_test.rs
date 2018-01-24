extern crate hyper;
extern crate serde_json;
extern crate tokio_core;
extern crate users_lib;
extern crate futures;

use std::sync::Arc;
use std::time::SystemTime;

use tokio_core::reactor::Core;

use users_lib::repos::users::UsersRepo;
use users_lib::repos::types::RepoFuture;
use users_lib::services::users::{UsersServiceImpl, UsersService};
use users_lib::models::user::{NewUser, UpdateUser, User, Gender, Provider};

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

    fn create(&self, payload: NewUser, provider_arg: Provider) -> RepoFuture<User> {
        let user = create_user(1, payload.email);
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

fn create_service () -> UsersServiceImpl<UsersRepoMock> {
    UsersServiceImpl { users_repo : Arc::new(MOCK) } 
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
        email: email,
        password: password,
    }
}

fn create_update_user(email: String) -> UpdateUser {
    UpdateUser {
        email: email,
        phone: None,
        first_name: None,
        last_name: None,
        middle_name: None,
        gender: Gender::Male,
        birthdate: None,
        last_login_at: SystemTime::now(),
    }
}

const MOCK : UsersRepoMock = UsersRepoMock{};
static MOCK_EMAIL: &'static str = "example@mail.com";
static MOCK_PASSWORD: &'static str = "password";


#[test]
fn test_get_user() {
    let service = create_service();
    let mut core = Core::new().unwrap();
    let work = service.get(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
}

#[test]
fn test_list() {
    let service = create_service();
    let mut core = Core::new().unwrap();
    let work = service.list(1, 5);
    let result = core.run(work).unwrap();
    assert_eq!(result.len(), 5);
}

#[test]
fn test_create_allready_existed() {
    let service = create_service();
    let mut core = Core::new().unwrap();
    let new_user = create_new_user (MOCK_EMAIL.to_string(), MOCK_PASSWORD.to_string());
    let work = service.create(new_user);
    let result = core.run(work);
    assert_eq!(result.is_err(), true);
}

#[test]
fn test_create_user() {
    let service = create_service();
    let mut core = Core::new().unwrap();
    let new_user = create_new_user ("new_user@mail.com".to_string(), MOCK_PASSWORD.to_string());
    let work = service.create(new_user);
    let result = core.run(work).unwrap();
    assert_eq!(result.email, "new_user@mail.com".to_string());
}

#[test]
fn test_update() {
    let service = create_service();
    let mut core = Core::new().unwrap();
    let update_user = create_update_user(MOCK_EMAIL.to_string());
    let work = service.update(1, update_user);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.email, MOCK_EMAIL.to_string());
}

#[test]
fn test_deactivate() {
    let service = create_service();
    let mut core = Core::new().unwrap();
    let work = service.deactivate(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.is_active, false);
}