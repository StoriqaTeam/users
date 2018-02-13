extern crate base64;
extern crate futures;
extern crate hyper;
extern crate rand;
extern crate serde_json;
extern crate sha3;
extern crate tokio_core;
extern crate users_lib;

use std::time::SystemTime;
use std::sync::{Arc, Mutex};

use tokio_core::reactor::Core;
use futures::Stream;
use sha3::{Digest, Sha3_256};
use base64::encode;

use users_lib::repos::users::UsersRepo;
use users_lib::repos::user_roles::UserRolesRepo;
use users_lib::repos::identities::IdentitiesRepo;
use users_lib::repos::types::RepoFuture;
use users_lib::services::users::{UsersService, UsersServiceImpl};
use users_lib::services::jwt::{JWTService, JWTServiceImpl};
use users_lib::models::user::{Gender, NewUser, UpdateUser, User};
use users_lib::models::identity::{Identity, NewIdentity, Provider};
use users_lib::models::user_role::{NewUserRole, UserRole};
use users_lib::models::authorization::Role;
use users_lib::models::jwt::ProviderOauth;
use users_lib::models::authorization::*;
use users_lib::config::Config;
use users_lib::http::client::{Client, ClientHandle};

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

    fn create(&self, payload: NewUser) -> RepoFuture<User> {
        let user = create_user(1, payload.email);
        Box::new(futures::future::ok(user))
    }

    fn update(&self, user_id: i32, _payload: UpdateUser) -> RepoFuture<User> {
        let user = create_user(user_id, MOCK_EMAIL.to_string());

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
        Box::new(futures::future::ok(
            email_arg == MOCK_EMAIL.to_string() && provider_arg == Provider::Email,
        ))
    }

    fn create(&self, email: String, password: Option<String>, provider_arg: Provider, user_id: i32) -> RepoFuture<Identity> {
        let ident = create_identity(email, password, user_id, provider_arg);
        Box::new(futures::future::ok(ident))
    }

    fn verify_password(&self, email_arg: String, password_arg: String) -> RepoFuture<bool> {
        Box::new(futures::future::ok(
            email_arg == MOCK_EMAIL.to_string() && password_arg == password_create(MOCK_PASSWORD.to_string()),
        ))
    }

    fn find_by_email_provider(&self, email_arg: String, provider_arg: Provider) -> RepoFuture<Identity> {
        let ident = create_identity(
            email_arg,
            Some(password_create(MOCK_PASSWORD.to_string())),
            1,
            provider_arg,
        );
        Box::new(futures::future::ok(ident))
    }
}

pub fn new_users_service(
    users_repo: UsersRepoMock,
    ident_repo: IdentitiesRepoMock,
    user_id: Option<i32>,
) -> UsersServiceImpl<UsersRepoMock, IdentitiesRepoMock, AclImplMock> {
    let cache = CachedRoles::new(user_roles_repo);
    let aclimpl = AclImpl::new(cache);
    let acl = AclImplMock::new(aclimpl);
    UsersServiceImpl {
        users_repo,
        ident_repo,
        acl,
        user_id,
    }
}

fn create_users_service(users_id: Option<i32>) -> UsersServiceImpl<UsersRepoMock, IdentitiesRepoMock, AclImplMock> {
    new_users_service(MOCK_USERS, MOCK_IDENT, MOCK_USER_ROLE, users_id)
}

pub fn new_jwt_service(
    users_repo: UsersRepoMock,
    ident_repo: IdentitiesRepoMock,
    http_client: ClientHandle,
    config: Config,
) -> JWTServiceImpl<UsersRepoMock, IdentitiesRepoMock> {
    JWTServiceImpl {
        users_repo: users_repo,
        ident_repo: ident_repo,
        http_client: http_client,
        google_config: config.google,
        facebook_config: config.facebook,
        jwt_config: config.jwt,
    }
}

fn create_jwt_service() -> (Core, JWTServiceImpl<UsersRepoMock, IdentitiesRepoMock>) {
    let config = Config::new().unwrap();
    let core = Core::new().expect("Unexpected error creating event loop core");
    let handle = Arc::new(core.handle());
    let client = Client::new(&config, &handle);
    let client_handle = client.handle();
    let client_stream = client.stream();
    handle.spawn(client_stream.for_each(|_| Ok(())));
    let service = new_jwt_service(MOCK_USERS, MOCK_IDENT, client_handle, config);
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
        updated_at: SystemTime::now(),
    }
}

fn create_new_identity(email: String, password: String) -> NewIdentity {
    NewIdentity {
        email: email,
        password: password,
    }
}

fn create_update_user(_email: String) -> UpdateUser {
    UpdateUser {
        phone: None,
        first_name: None,
        last_name: None,
        middle_name: None,
        gender: None,
        birthdate: None,
        last_login_at: Some(SystemTime::now()),
    }
}

fn create_identity(email: String, password: Option<String>, user_id: i32, provider: Provider) -> Identity {
    Identity {
        user_email: email,
        user_password: password,
        user_id: user_id,
        provider: provider,
    }
}

fn password_create(clear_password: String) -> String {
    let salt = rand::random::<u64>().to_string().split_off(10);
    let pass = clear_password + &salt;
    let mut hasher = Sha3_256::default();
    hasher.input(pass.as_bytes());
    let out = hasher.result();
    let computed_hash = encode(&out[..]);
    computed_hash + "." + &salt
}

const MOCK_USERS: UsersRepoMock = UsersRepoMock {};
const MOCK_IDENT: IdentitiesRepoMock = IdentitiesRepoMock {};
static MOCK_EMAIL: &'static str = "example@mail.com";
static MOCK_PASSWORD: &'static str = "password";
static GOOGLE_TOKEN: &'static str =
    "ya29.GlxRBXyOU1dfRmFEdVE1oOK3SyQ6UKh4RTESu0J-C19N2o5RCQVEALMi5DKlgctjTQclLCrLQkUovOb05ikfYQdZ2paFja9Uf4GN1hoysgp_dDr9NLgvfo7fGthY8A";
static FACEBOOK_TOKEN: &'static str = "AQDr-FG4bmYyrhYGk9ZJg1liqTRBfKfRbXopSd72_Qjexg3e4ybh9EJZFErHwyhw0oKyUOEbCQSalC4D8b3B2r4eJiyEmyW-E_ESsVnyThn27j8KEDDfsxCwUJxZY6fDwZt9LWMEHnHYEnFxABIupKN8y8bj_SH8wxIZoDm-YzZtYbj7VUf9g0vPKOkA_1hnjjW8TGrEKmbhFZLWLj6wJgC3uek3D3MahUhd_k3K-4BjOJNyXa8h_ESPQWNHt9sIIIDmhAw5X4iVmdbte7tQWf6y96vd_muwA4hKMRxzc7gMQo16tcI7hazQaJ1rJj39G8poG9Ac7AjdO6O7vSnYB9IqeLFbhKH56IyJoCR_05e2tg";
