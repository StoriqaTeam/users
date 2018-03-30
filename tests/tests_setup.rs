extern crate base64;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate rand;
extern crate serde_json;
extern crate sha3;
extern crate tokio_core;
extern crate users_lib;
extern crate diesel;
extern crate r2d2;

use std::time::SystemTime;
use std::sync::Arc;
use std::error::Error;
use std::fmt;

use base64::encode;
use futures::Stream;
use futures_cpupool::CpuPool;
use sha3::{Digest, Sha3_256};
use tokio_core::reactor::Core;

use r2d2::ManageConnection;

use diesel::Connection;
use diesel::ConnectionResult;
use diesel::QueryResult;
use diesel::query_builder::AsQuery;
use diesel::query_builder::QueryFragment;
use diesel::pg::Pg;
use diesel::query_builder::QueryId;
use diesel::sql_types::HasSqlType;
use diesel::Queryable;
use diesel::deserialize::QueryableByName;
use diesel::connection::AnsiTransactionManager;
use diesel::connection::SimpleConnection;

use users_lib::config::Config;
use users_lib::stq_http::client::{Client, ClientHandle};
use users_lib::models::*;
use users_lib::models::authorization::*;
use users_lib::repos::repo_factory::ReposFactory;
use users_lib::repos::error::RepoError;
use users_lib::repos::users::UsersRepo;
use users_lib::repos::identities::IdentitiesRepo;
use users_lib::repos::reset_token::ResetTokenRepo;
use users_lib::repos::user_roles::UserRolesRepo;
use users_lib::services::jwt::{JWTService, JWTServiceImpl};
use users_lib::services::users::{UsersService, UsersServiceImpl};

#[derive(Default, Copy, Clone)]
pub struct ReposFactoryMock;

impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryMock {
    fn create_users_repo<'a>(&self, db_conn: &'a C, user_id: Option<i32>) -> Box<UsersRepo + 'a> {
        Box::new(UsersRepoMock::default()) as Box<UsersRepo>
    }

    fn create_users_repo_with_sys_acl<'a>(&self, db_conn: &'a C) -> Box<UsersRepo + 'a> {
        Box::new(UsersRepoMock::default()) as Box<UsersRepo>
    }

    fn create_identities_repo<'a>(&self, db_conn: &'a C) -> Box<IdentitiesRepo + 'a> {
        Box::new(IdentitiesRepoMock::default()) as Box<IdentitiesRepo>
    }

    fn create_reset_token_repo<'a>(&self, db_conn: &'a C) -> Box<ResetTokenRepo + 'a> {
        Box::new(ResetTokenRepoMock::default()) as Box<ResetTokenRepo>
    }

    fn create_user_roles_repo<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
        Box::new(UserRolesRepoMock::default()) as Box<UserRolesRepo>
    }
}

#[derive(Clone, Default)]
pub struct UsersRepoMock;

impl UsersRepo for UsersRepoMock {
    fn find(&self, user_id: i32) -> Result<User, RepoError> {
        let user = create_user(user_id, MOCK_EMAIL.to_string());
        Ok(user)
    }

    fn email_exists(&self, email_arg: String) -> Result<bool, RepoError> {
        Ok(email_arg == MOCK_EMAIL.to_string())
    }

    fn find_by_email(&self, email_arg: String) -> Result<User, RepoError> {
        let user = create_user(1, email_arg);
        Ok(user)
    }

    fn list(&self, from: i32, count: i64) -> Result<Vec<User>, RepoError> {
        let mut users = vec![];
        for i in from..(from + count as i32) {
            let user = create_user(i, MOCK_EMAIL.to_string());
            users.push(user);
        }
        Ok(users)
    }

    fn create(&self, payload: NewUser) -> Result<User, RepoError> {
        let user = create_user(1, payload.email);
        Ok(user)
    }

    fn update(&self, user_id: i32, _payload: UpdateUser) -> Result<User, RepoError> {
        let user = create_user(user_id, MOCK_EMAIL.to_string());
        Ok(user)
    }

    fn deactivate(&self, user_id: i32) -> Result<User, RepoError> {
        let mut user = create_user(user_id, MOCK_EMAIL.to_string());
        user.is_active = false;
        Ok(user)
    }

    fn delete_by_saga_id(&mut self, saga_id_arg: String) -> Result<User, RepoError> {
        let user = create_user(1, payload.email);
        Ok(user)
    }
}

#[derive(Clone, Default)]
pub struct IdentitiesRepoMock;

impl IdentitiesRepo for IdentitiesRepoMock {
    fn email_provider_exists(&self, email_arg: String, provider_arg: Provider) -> Result<bool, RepoError> {
        Ok(email_arg == MOCK_EMAIL.to_string() && provider_arg == Provider::Email)
    }

    fn create(&self, email: String, password: Option<String>, provider_arg: Provider, user_id: i32, saga_id: String) -> Result<Identity, RepoError> {
        let ident = create_identity(email, password, user_id, provider_arg, MOCK_SAGA_ID);
        Ok(ident)
    }

    fn verify_password(&self, email_arg: String, password_arg: String) -> Result<bool, RepoError> {
        Box::new(futures::future::ok(
            email_arg == MOCK_EMAIL.to_string() && password_arg == password_create(MOCK_PASSWORD.to_string()),
        ))
    }

    fn find_by_email_provider(&self, email_arg: String, provider_arg: Provider) -> Result<Identity, RepoError> {
        let ident = create_identity(
            email_arg,
            Some(password_create(MOCK_PASSWORD.to_string())),
            1,
            provider_arg,
            MOCK_SAGA_ID
        );
        Ok(ident)
    }

    fn update(&self, ident: Identity, update: UpdateIdentity) -> Result<Identity, RepoError> {
        let ident = create_identity(
            email_arg,
            update.password,
            1,
            provider_arg,
            MOCK_SAGA_ID
        );
        Ok(ident)
    }
}

#[derive(Clone, Default)]
pub struct ResetTokenRepoMock;

impl ResetTokenRepo for ResetTokenRepoMock {
    /// Create token for user
    fn create(&self, reset_token_arg: ResetToken) -> Result<ResetToken, RepoError> {
        let token = create_reset_token(
            MOCK_TOKEN.to_string(),
            MOCK_EMAIL.to_string()
        );

        Ok(token)
    }

    /// Find by token
    fn find_by_token(&self, token_arg: String) -> Result<ResetToken, RepoError> {
        let token = create_reset_token(
            MOCK_TOKEN.to_string(),
            MOCK_EMAIL.to_string()
        );

        Ok(token)
    }

    /// Find by email
    fn find_by_email(&self, email_arg: String) -> Result<ResetToken, RepoError> {
        let token = create_reset_token(
            MOCK_TOKEN.to_string(),
            MOCK_EMAIL.to_string()
        );

        Ok(token)
    }

    /// Delete by token
    fn delete_by_token(&self, token_arg: String) -> Result<ResetToken, RepoError> {
        let token = create_reset_token(
            MOCK_TOKEN.to_string(),
            MOCK_EMAIL.to_string()
        );

        Ok(token)
    }

    /// Delete by email
    fn delete_by_email(&self, email_arg: String) -> Result<ResetToken, RepoError> {
        let token = create_reset_token(
            MOCK_TOKEN.to_string(),
            MOCK_EMAIL.to_string()
        );

        Ok(token)
    }
}

#[derive(Clone, Default)]
pub struct UserRolesRepoMock;

impl UserRolesRepo for UserRolesRepoMock {
    fn list_for_user(&self, user_id_value: i32) -> Result<Vec<Role>, RepoError> {
        Ok(match user_id_value {
            1 => vec![Role::Superuser],
            _ => vec![Role::User],
        })
    }

    fn create(&self, payload: NewUserRole) -> Result<UserRole, RepoError> {
        Ok(UserRole {
            id: 123,
            user_id: payload.user_id,
            role: payload.role,
        })
    }

    fn delete(&self, payload: OldUserRole) -> Result<UserRole, RepoError> {
        Ok(UserRole {
            id: 123,
            user_id: payload.user_id,
            role: payload.role,
        })
    }

    fn delete_by_user_id(&self, user_id_arg: i32) -> Result<UserRole, RepoError> {
        Ok(UserRole {
            id: 123,
            user_id: user_id_arg,
            role: Role::User,
        })
    }
}

pub fn new_users_service(
    users_repo: UsersRepoMock,
    ident_repo: IdentitiesRepoMock,
    user_id: Option<i32>,
) -> UsersServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
    let manager = MockConnectionManager::default();
    let db_pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create connection pool");
    let cpu_pool = CpuPool::new(1);

    let config = Config::new().unwrap();
    let http_config = HttpConfig {
        http_client_retries: config.client.http_client_retries,
        http_client_buffer_size: config.client.http_client_buffer_size,
    };
    let client = stq_http::client::Client::new(&http_config, &handle);
    let client_handle = client.handle();

    UsersServiceImpl {
        db_pool: db_pool,
        cpu_pool: cpu_pool,
        http_client: client_handle,
        user_id: user_id,
        notif_conf: NOTIF_CONFIG_MOCK,
        repo_factory: MOCK_REPO_FACTORY,
    }
}

fn create_users_service(users_id: Option<i32>) -> UsersServiceImpl<UsersRepoMock, IdentitiesRepoMock, AclImplMock> {
    new_users_service(MOCK_USERS, MOCK_IDENT, users_id)
}

pub fn new_jwt_service(
    users_repo: UsersRepoMock,
    ident_repo: IdentitiesRepoMock,
    http_client: ClientHandle,
    config: Config,
) -> JWTServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
    let manager = MockConnectionManager::default();
    let db_pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create connection pool");
    let cpu_pool = CpuPool::new(1);

    let config = Config::new().unwrap();
    let http_config = HttpConfig {
        http_client_retries: config.client.http_client_retries,
        http_client_buffer_size: config.client.http_client_buffer_size,
    };
    let client = stq_http::client::Client::new(&http_config, &handle);
    let client_handle = client.handle();

    JWTServiceImpl {
        db_pool: db_pool,
        cpu_pool: cpu_pool,
        http_client: client_handle,
        saga_addr: "saga_addr",
        google_config: config.google,
        facebook_config: config.facebook,
        jwt_config: config.jwt,
        repo_factory: MOCK_REPO_FACTORY,
    }
}

fn create_jwt_service() -> (Core, JWTServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock>) {
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
        saga_id: MOCK_SAGA_ID.to_string(),
    }
}

fn create_new_identity(email: String, password: String, provider: Provider, saga_id: String) -> NewIdentity {
    NewIdentity {
        email,
        password,
        provider,
        saga_id,
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
        is_active: None,
        email_verified: None,
    }
}

fn create_identity(email: String, password: Option<String>, user_id: i32, provider: Provider, saga_id: String) -> Identity {
    Identity {
        email,
        password,
        user_id,
        provider,
        saga_id
    }
}

fn create_reset_token(token: String, email: String) -> ResetToken {
    ResetToken {
        token,
        email,
        created_at: SystemTime::now(),
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

#[derive(Default)]
struct MockConnection {
    tr: AnsiTransactionManager,
}

impl Connection for MockConnection {
    type Backend = Pg;
    type TransactionManager = AnsiTransactionManager;

    fn establish(_database_url: &str) -> ConnectionResult<MockConnection> {
        Ok(MockConnection::default())
    }

    fn execute(&self, _query: &str) -> QueryResult<usize> {
        unimplemented!()
    }

    fn query_by_index<T, U>(&self, _source: T) -> QueryResult<Vec<U>>
        where
            T: AsQuery,
            T::Query: QueryFragment<Pg> + QueryId,
            Pg: HasSqlType<T::SqlType>,
            U: Queryable<T::SqlType, Pg>,
    {
        unimplemented!()
    }

    fn query_by_name<T, U>(&self, _source: &T) -> QueryResult<Vec<U>>
        where
            T: QueryFragment<Pg> + QueryId,
            U: QueryableByName<Pg>,
    {
        unimplemented!()
    }

    fn execute_returning_count<T>(&self, _source: &T) -> QueryResult<usize>
        where
            T: QueryFragment<Pg> + QueryId,
    {
        unimplemented!()
    }

    fn transaction_manager(&self) -> &Self::TransactionManager {
        &self.tr
    }
}

impl SimpleConnection for MockConnection {
    fn batch_execute(&self, _query: &str) -> QueryResult<()> {
        Ok(())
    }
}

#[derive(Default)]
struct MockConnectionManager;

impl ManageConnection for MockConnectionManager {
    type Connection = MockConnection;
    type Error = MockError;

    fn connect(&self) -> Result<MockConnection, MockError> {
        Ok(MockConnection::default())
    }

    fn is_valid(&self, _conn: &mut MockConnection) -> Result<(), MockError> {
        Ok(())
    }

    fn has_broken(&self, _conn: &mut MockConnection) -> bool {
        false
    }
}

#[derive(Debug)]
struct MockError {}

impl fmt::Display for MockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SuperError is here!")
    }
}

impl Error for MockError {
    fn description(&self) -> &str {
        "I'm the superhero of errors"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

const MOCK_REPO_FACTORY: ReposFactoryMock = ReposFactoryMock {};
const MOCK_USERS: UsersRepoMock = UsersRepoMock {};
const MOCK_IDENT: IdentitiesRepoMock = IdentitiesRepoMock {};
static MOCK_EMAIL: &'static str = "example@mail.com";
static MOCK_PASSWORD: &'static str = "password";
static MOCK_TOKEN: &'static str = "token";
static MOCK_SAGA_ID: &'static str = "saga_id";
static GOOGLE_TOKEN: &'static str =
    "ya29.GlxRBXyOU1dfRmFEdVE1oOK3SyQ6UKh4RTESu0J-C19N2o5RCQVEALMi5DKlgctjTQclLCrLQkUovOb05ikfYQdZ2paFja9Uf4GN1hoysgp_dDr9NLgvfo7fGthY8A";
static FACEBOOK_TOKEN: &'static str = "AQDr-FG4bmYyrhYGk9ZJg1liqTRBfKfRbXopSd72_Qjexg3e4ybh9EJZFErHwyhw0oKyUOEbCQSalC4D8b3B2r4eJiyEmyW-E_ESsVnyThn27j8KEDDfsxCwUJxZY6fDwZt9LWMEHnHYEnFxABIupKN8y8bj_SH8wxIZoDm-YzZtYbj7VUf9g0vPKOkA_1hnjjW8TGrEKmbhFZLWLj6wJgC3uek3D3MahUhd_k3K-4BjOJNyXa8h_ESPQWNHt9sIIIDmhAw5X4iVmdbte7tQWf6y96vd_muwA4hKMRxzc7gMQo16tcI7hazQaJ1rJj39G8poG9Ac7AjdO6O7vSnYB9IqeLFbhKH56IyJoCR_05e2tg";
