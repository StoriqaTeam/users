extern crate futures;
extern crate hyper;
extern crate serde_json;
extern crate tokio_core;
extern crate users_lib;

use std::time::SystemTime;

use tokio_core::reactor::Core;

use users_lib::repos::users::UsersRepo;
use users_lib::repos::identities::IdentitiesRepo;
use users_lib::repos::types::RepoFuture;
use users_lib::services::users::{UsersService, UsersServiceImpl};
use users_lib::models::user::{Gender, Identity, NewUser, Provider, UpdateUser, User};

#[derive(Clone)]
pub struct UsersRepoMock;

impl UsersRepo for UsersRepoMock {
    fn find(&self, user_id: i32) -> RepoFuture<User> {
        let user = create_user(user_id, MOCK_EMAIL.to_string());
        Box::new(futures::future::ok(user))
    }

    fn email_exists(&self, email_arg: String) -> RepoFuture<bool> {
        Box::new(futures::future::ok(email_arg == MOCK_EMAIL.to_string()))
    }

    fn find_by_email(&self, email_arg: String) -> RepoFuture<User> {
        let user = create_user(1, email_arg);
        Box::new(futures::future::ok(user))
    }

    fn list(&self, from: i32, count: i64) -> RepoFuture<Vec<User>> {
        let mut users = vec![];
        for i in from..(from + count as i32) {
            let user = create_user(i, MOCK_EMAIL.to_string());
            users.push(user);
        }
        Box::new(futures::future::ok(users))
    }

    fn create(&self, payload: UpdateUser) -> RepoFuture<User> {
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
}


#[derive(Clone)]
pub struct IdentitiesRepoMock;

impl IdentitiesRepo for IdentitiesRepoMock {
    fn email_provider_exists(&self, email_arg: String, provider_arg: Provider) -> RepoFuture<bool> {
        Box::new(futures::future::ok(email_arg == MOCK_EMAIL.to_string() && provider_arg == Provider::Email))
    }

    fn create(
        &self,
        email: String,
        password: Option<String>,
        provider_arg: Provider,
        user_id: i32,
    ) -> RepoFuture<Identity> {
        let ident = create_identity(email, password, user_id, provider_arg);
        Box::new(futures::future::ok(ident))
    }

    fn verify_password(&self, email_arg: String, password_arg: String) -> RepoFuture<bool> {
        Box::new(futures::future::ok(
            email_arg == MOCK_EMAIL.to_string() && password_arg == MOCK_PASSWORD.to_string(),
        ))
    }
}




pub fn new_service(
    users_repo: UsersRepoMock,
    ident_repo: IdentitiesRepoMock,
    user_email: Option<String>,
) -> UsersServiceImpl<UsersRepoMock, IdentitiesRepoMock> {
    UsersServiceImpl {
        users_repo,
        ident_repo,
        user_email,
    }
}

fn create_service(
    users_email: Option<String>,
) -> UsersServiceImpl<UsersRepoMock, IdentitiesRepoMock> {
    new_service(MOCK_USERS, MOCK_IDENT, users_email)
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
        updated_at: SystemTime::now(),
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

fn create_identity(
    email: String,
    password: Option<String>,
    user_id: i32, 
    provider: Provider) 
    -> Identity {
        Identity {
            user_email: email,
            user_password: password,
            user_id: user_id,
            provider: provider,
        }
}

const MOCK_USERS: UsersRepoMock = UsersRepoMock {};
const MOCK_IDENT: IdentitiesRepoMock = IdentitiesRepoMock {};
static MOCK_EMAIL: &'static str = "example@mail.com";
static MOCK_PASSWORD: &'static str = "password";


#[test]
fn test_get_user() {
    let service = create_service(Some(MOCK_EMAIL.to_string()));
    let mut core = Core::new().unwrap();
    let work = service.get(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
}

#[test]
fn test_current_user() {
    let service = create_service(Some(MOCK_EMAIL.to_string()));
    let mut core = Core::new().unwrap();
    let work = service.current();
    let result = core.run(work).unwrap();
    assert_eq!(result.email, MOCK_EMAIL.to_string());
}

#[test]
fn test_current_user_without_user_email() {
    let service = create_service(None);
    let mut core = Core::new().unwrap();
    let work = service.current();
    let result = core.run(work);
    assert_eq!(result.is_err(), true);
}

#[test]
fn test_list() {
    let service = create_service(Some(MOCK_EMAIL.to_string()));
    let mut core = Core::new().unwrap();
    let work = service.list(1, 5);
    let result = core.run(work).unwrap();
    assert_eq!(result.len(), 5);
}

#[test]
fn test_create_allready_existed() {
    let service = create_service(Some(MOCK_EMAIL.to_string()));
    let mut core = Core::new().unwrap();
    let new_user = create_new_user(MOCK_EMAIL.to_string(), MOCK_PASSWORD.to_string());
    let work = service.create(new_user);
    let result = core.run(work);
    assert_eq!(result.is_err(), true);
}

#[test]
fn test_create_user() {
    let service = create_service(Some(MOCK_EMAIL.to_string()));
    let mut core = Core::new().unwrap();
    let new_user = create_new_user("new_user@mail.com".to_string(), MOCK_PASSWORD.to_string());
    let work = service.create(new_user);
    let result = core.run(work).unwrap();
    assert_eq!(result.email, "new_user@mail.com".to_string());
}

#[test]
fn test_update() {
    let service = create_service(Some(MOCK_EMAIL.to_string()));
    let mut core = Core::new().unwrap();
    let update_user = create_update_user(MOCK_EMAIL.to_string());
    let work = service.update(1, update_user);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.email, MOCK_EMAIL.to_string());
}

#[test]
fn test_deactivate() {
    let service = create_service(Some(MOCK_EMAIL.to_string()));
    let mut core = Core::new().unwrap();
    let work = service.deactivate(1);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, 1);
    assert_eq!(result.is_active, false);
}
