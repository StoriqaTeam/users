use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;

use stq_types::{UserId, UsersRole};

use models::*;
use repos::legacy_acl::{Acl, SystemACL, UnauthorizedACL};
use repos::*;

pub trait ReposFactory<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>:
    Clone + Send + Sync + 'static
{
    fn create_users_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<UsersRepo + 'a>;
    fn create_users_repo_with_sys_acl<'a>(&self, db_conn: &'a C) -> Box<UsersRepo + 'a>;
    fn create_identities_repo<'a>(&self, db_conn: &'a C) -> Box<IdentitiesRepo + 'a>;
    fn create_reset_token_repo<'a>(&self, db_conn: &'a C) -> Box<ResetTokenRepo + 'a>;
    fn create_user_roles_repo_with_sys_acl<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a>;
    fn create_user_roles_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<UserRolesRepo + 'a>;
}

#[derive(Clone)]
pub struct ReposFactoryImpl {
    roles_cache: RolesCacheImpl,
}

impl ReposFactoryImpl {
    pub fn new(roles_cache: RolesCacheImpl) -> Self {
        Self { roles_cache }
    }

    pub fn get_roles<'a, C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>(
        &self,
        id: UserId,
        db_conn: &'a C,
    ) -> Vec<UsersRole> {
        self.create_user_roles_repo_with_sys_acl(db_conn)
            .list_for_user(id)
            .ok()
            .unwrap_or_default()
    }

    fn get_acl<'a, T, C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static>(
        &self,
        db_conn: &'a C,
        user_id: Option<UserId>,
    ) -> Box<Acl<Resource, Action, Scope, FailureError, T>> {
        user_id.map_or(
            Box::new(UnauthorizedACL::default()) as Box<Acl<Resource, Action, Scope, FailureError, T>>,
            |id| {
                let roles = self.get_roles(id, db_conn);
                (Box::new(ApplicationAcl::new(roles, id)) as Box<Acl<Resource, Action, Scope, FailureError, T>>)
            },
        )
    }
}

impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryImpl {
    fn create_users_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<UsersRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(UsersRepoImpl::new(db_conn, acl)) as Box<UsersRepo>
    }

    fn create_users_repo_with_sys_acl<'a>(&self, db_conn: &'a C) -> Box<UsersRepo + 'a> {
        Box::new(UsersRepoImpl::new(
            db_conn,
            Box::new(SystemACL::default()) as Box<Acl<Resource, Action, Scope, FailureError, User>>,
        )) as Box<UsersRepo>
    }

    fn create_identities_repo<'a>(&self, db_conn: &'a C) -> Box<IdentitiesRepo + 'a> {
        Box::new(IdentitiesRepoImpl::new(db_conn)) as Box<IdentitiesRepo>
    }

    fn create_reset_token_repo<'a>(&self, db_conn: &'a C) -> Box<ResetTokenRepo + 'a> {
        Box::new(ResetTokenRepoImpl::new(db_conn)) as Box<ResetTokenRepo>
    }

    fn create_user_roles_repo_with_sys_acl<'a>(&self, db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
        Box::new(UserRolesRepoImpl::new(
            db_conn,
            Box::new(SystemACL::default()) as Box<Acl<Resource, Action, Scope, FailureError, UserRole>>,
            self.roles_cache.clone(),
        )) as Box<UserRolesRepo>
    }

    fn create_user_roles_repo<'a>(&self, db_conn: &'a C, user_id: Option<UserId>) -> Box<UserRolesRepo + 'a> {
        let acl = self.get_acl(db_conn, user_id);
        Box::new(UserRolesRepoImpl::new(db_conn, acl, self.roles_cache.clone())) as Box<UserRolesRepo>
    }
}

#[cfg(test)]
pub mod tests {
    extern crate base64;
    extern crate diesel;
    extern crate futures;
    extern crate futures_cpupool;
    extern crate hyper;
    extern crate r2d2;
    extern crate rand;
    extern crate serde_json;
    extern crate sha3;
    extern crate stq_http;
    extern crate tokio_core;

    use std::error::Error;
    use std::fmt;
    use std::fs::File;
    use std::io::prelude::*;
    use std::sync::Arc;
    use std::time::SystemTime;

    use base64::encode;
    use diesel::connection::AnsiTransactionManager;
    use diesel::connection::SimpleConnection;
    use diesel::deserialize::QueryableByName;
    use diesel::pg::Pg;
    use diesel::query_builder::AsQuery;
    use diesel::query_builder::QueryFragment;
    use diesel::query_builder::QueryId;
    use diesel::sql_types::HasSqlType;
    use diesel::Connection;
    use diesel::ConnectionResult;
    use diesel::QueryResult;
    use diesel::Queryable;
    use futures_cpupool::CpuPool;
    use r2d2::ManageConnection;
    use sha3::{Digest, Sha3_256};
    use tokio_core::reactor::Handle;

    use stq_static_resources::{Provider, TokenType};
    use stq_types::{UserId, UsersRole};

    use config::Config;
    use models::*;
    use repos::identities::IdentitiesRepo;
    use repos::repo_factory::ReposFactory;
    use repos::reset_token::ResetTokenRepo;
    use repos::types::RepoResult;
    use repos::user_roles::UserRolesRepo;
    use repos::users::UsersRepo;
    use services::jwt::JWTServiceImpl;
    use services::users::UsersServiceImpl;

    #[derive(Default, Copy, Clone)]
    pub struct ReposFactoryMock;

    impl<C: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ReposFactory<C> for ReposFactoryMock {
        fn create_users_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<UsersRepo + 'a> {
            Box::new(UsersRepoMock::default()) as Box<UsersRepo>
        }

        fn create_users_repo_with_sys_acl<'a>(&self, _db_conn: &'a C) -> Box<UsersRepo + 'a> {
            Box::new(UsersRepoMock::default()) as Box<UsersRepo>
        }

        fn create_identities_repo<'a>(&self, _db_conn: &'a C) -> Box<IdentitiesRepo + 'a> {
            Box::new(IdentitiesRepoMock::default()) as Box<IdentitiesRepo>
        }

        fn create_reset_token_repo<'a>(&self, _db_conn: &'a C) -> Box<ResetTokenRepo + 'a> {
            Box::new(ResetTokenRepoMock::default()) as Box<ResetTokenRepo>
        }

        fn create_user_roles_repo<'a>(&self, _db_conn: &'a C, _user_id: Option<UserId>) -> Box<UserRolesRepo + 'a> {
            Box::new(UserRolesRepoMock::default()) as Box<UserRolesRepo>
        }

        fn create_user_roles_repo_with_sys_acl<'a>(&self, _db_conn: &'a C) -> Box<UserRolesRepo + 'a> {
            Box::new(UserRolesRepoMock::default()) as Box<UserRolesRepo>
        }
    }

    #[derive(Clone, Default)]
    pub struct UsersRepoMock;

    impl UsersRepo for UsersRepoMock {
        fn find(&self, user_id: UserId) -> RepoResult<Option<User>> {
            let user = create_user(user_id, MOCK_EMAIL.to_string());
            Ok(Some(user))
        }

        fn email_exists(&self, email_arg: String) -> RepoResult<bool> {
            Ok(email_arg == MOCK_EMAIL.to_string())
        }

        fn find_by_email(&self, email_arg: String) -> RepoResult<Option<User>> {
            let user = create_user(UserId(1), email_arg);
            Ok(Some(user))
        }

        fn list(&self, from: i32, count: i64) -> RepoResult<Vec<User>> {
            let mut users = vec![];
            for i in from..(from + count as i32) {
                let user = create_user(UserId(i), MOCK_EMAIL.to_string());
                users.push(user);
            }
            Ok(users)
        }

        fn create(&self, payload: NewUser) -> RepoResult<User> {
            let user = create_user(UserId(1), payload.email);
            Ok(user)
        }

        fn update(&self, user_id: UserId, _payload: UpdateUser) -> RepoResult<User> {
            let user = create_user(user_id, MOCK_EMAIL.to_string());
            Ok(user)
        }

        fn deactivate(&self, user_id: UserId) -> RepoResult<User> {
            let mut user = create_user(user_id, MOCK_EMAIL.to_string());
            user.is_active = false;
            Ok(user)
        }

        fn delete_by_saga_id(&self, _saga_id_arg: String) -> RepoResult<User> {
            let user = create_user(UserId(1), MOCK_EMAIL.to_string());
            Ok(user)
        }
    }

    #[derive(Clone, Default)]
    pub struct IdentitiesRepoMock;

    impl IdentitiesRepo for IdentitiesRepoMock {
        fn email_exists(&self, email_arg: String) -> RepoResult<bool> {
            Ok(email_arg == MOCK_EMAIL.to_string())
        }

        fn email_provider_exists(&self, email_arg: String, provider_arg: Provider) -> RepoResult<bool> {
            Ok(email_arg == MOCK_EMAIL.to_string() && provider_arg == Provider::Email)
        }

        fn create(
            &self,
            email: String,
            password: Option<String>,
            provider_arg: Provider,
            user_id: UserId,
            _saga_id: String,
        ) -> RepoResult<Identity> {
            let ident = create_identity(email, password, user_id, provider_arg, MOCK_SAGA_ID.to_string());
            Ok(ident)
        }

        fn verify_password(&self, email_arg: String, password_arg: String) -> RepoResult<bool> {
            Ok(email_arg == MOCK_EMAIL.to_string() && password_arg == password_create(MOCK_PASSWORD.to_string()))
        }

        fn find_by_email_provider(&self, email_arg: String, provider_arg: Provider) -> RepoResult<Identity> {
            let ident = create_identity(
                email_arg,
                Some(password_create(MOCK_PASSWORD.to_string())),
                UserId(1),
                provider_arg,
                MOCK_SAGA_ID.to_string(),
            );
            Ok(ident)
        }

        fn find_by_id_provider(&self, user_id: UserId, provider_arg: Provider) -> RepoResult<Identity> {
            let ident = create_identity(
                MOCK_EMAIL.to_string(),
                Some(password_create(MOCK_PASSWORD.to_string())),
                UserId(user_id.0),
                provider_arg,
                MOCK_SAGA_ID.to_string(),
            );
            Ok(ident)
        }

        fn update(&self, ident: Identity, update: UpdateIdentity) -> RepoResult<Identity> {
            let ident = create_identity(ident.email, update.password, UserId(1), ident.provider, ident.saga_id);
            Ok(ident)
        }
    }

    #[derive(Clone, Default)]
    pub struct ResetTokenRepoMock;

    impl ResetTokenRepo for ResetTokenRepoMock {
        /// Create token for user
        fn create(&self, _reset_token_arg: ResetToken) -> RepoResult<ResetToken> {
            let token = create_reset_token(MOCK_TOKEN.to_string(), MOCK_EMAIL.to_string());

            Ok(token)
        }

        /// Find by token
        fn find_by_token(&self, _token_arg: String, _token_type_arg: TokenType) -> RepoResult<ResetToken> {
            let token = create_reset_token(MOCK_TOKEN.to_string(), MOCK_EMAIL.to_string());

            Ok(token)
        }

        /// Find by email
        fn find_by_email(&self, _email_arg: String, _token_type_arg: TokenType) -> RepoResult<ResetToken> {
            let token = create_reset_token(MOCK_TOKEN.to_string(), MOCK_EMAIL.to_string());

            Ok(token)
        }

        /// Delete by token
        fn delete_by_token(&self, _token_arg: String, _token_type_arg: TokenType) -> RepoResult<ResetToken> {
            let token = create_reset_token(MOCK_TOKEN.to_string(), MOCK_EMAIL.to_string());

            Ok(token)
        }

        /// Delete by email
        fn delete_by_email(&self, _email_arg: String, _token_type_arg: TokenType) -> RepoResult<ResetToken> {
            let token = create_reset_token(MOCK_TOKEN.to_string(), MOCK_EMAIL.to_string());

            Ok(token)
        }
    }

    #[derive(Clone, Default)]
    pub struct UserRolesRepoMock;

    impl UserRolesRepo for UserRolesRepoMock {
        fn list_for_user(&self, user_id_value: UserId) -> RepoResult<Vec<UsersRole>> {
            Ok(match user_id_value.0 {
                1 => vec![UsersRole::Superuser],
                _ => vec![UsersRole::User],
            })
        }

        fn create(&self, payload: NewUserRole) -> RepoResult<UserRole> {
            Ok(UserRole {
                id: 123,
                user_id: payload.user_id,
                role: payload.role,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
            })
        }

        fn delete(&self, payload: OldUserRole) -> RepoResult<UserRole> {
            Ok(UserRole {
                id: 123,
                user_id: payload.user_id,
                role: payload.role,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
            })
        }

        fn delete_by_user_id(&self, user_id_arg: UserId) -> RepoResult<UserRole> {
            Ok(UserRole {
                id: 123,
                user_id: user_id_arg,
                role: UsersRole::User,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
            })
        }
    }

    pub fn create_users_service(
        user_id: Option<UserId>,
        handle: Arc<Handle>,
    ) -> UsersServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        let config = Config::new().unwrap();
        let client = stq_http::client::Client::new(&config.to_http_config(), &handle);
        let client_handle = client.handle();

        UsersServiceImpl::new(db_pool, cpu_pool, client_handle, user_id, MOCK_REPO_FACTORY)
    }

    pub fn create_jwt_service(handle: Arc<Handle>) -> JWTServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        let config = Config::new().unwrap();
        let client = stq_http::client::Client::new(&config.to_http_config(), &handle);
        let client_handle = client.handle();

        debug!("Reading private key file {}", &config.jwt.secret_key_path);
        let mut f = File::open(config.jwt.secret_key_path.clone()).unwrap();
        let mut jwt_private_key: Vec<u8> = Vec::new();
        f.read_to_end(&mut jwt_private_key).unwrap();

        JWTServiceImpl::new(db_pool, cpu_pool, client_handle, config, MOCK_REPO_FACTORY, jwt_private_key.clone())
    }

    pub fn create_user(id: UserId, email: String) -> User {
        User {
            id: id,
            email: email,
            email_verified: true,
            phone: None,
            phone_verified: false,
            is_active: true,
            first_name: None,
            last_name: None,
            middle_name: None,
            gender: None,
            avatar: None,
            birthdate: None,
            last_login_at: SystemTime::now(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            saga_id: MOCK_SAGA_ID.to_string(),
            is_blocked: false,
        }
    }

    pub fn create_new_identity(email: String, password: String, provider: Provider, saga_id: String) -> NewIdentity {
        NewIdentity {
            email,
            password: Some(password),
            provider,
            saga_id,
        }
    }

    pub fn create_new_email_identity(email: String, password: String) -> NewEmailIdentity {
        NewEmailIdentity { email, password }
    }

    pub fn create_update_user(_email: String) -> UpdateUser {
        UpdateUser {
            phone: None,
            first_name: None,
            last_name: None,
            middle_name: None,
            gender: None,
            birthdate: None,
            avatar: None,
            is_active: None,
            email_verified: None,
        }
    }

    pub fn create_identity(email: String, password: Option<String>, user_id: UserId, provider: Provider, saga_id: String) -> Identity {
        Identity {
            email,
            password,
            user_id,
            provider,
            saga_id,
        }
    }

    pub fn create_reset_token(token: String, email: String) -> ResetToken {
        ResetToken {
            token,
            email,
            token_type: TokenType::EmailVerify,
            created_at: SystemTime::now(),
        }
    }

    pub fn password_create(clear_password: String) -> String {
        let salt = rand::random::<u64>().to_string().split_off(10);
        let pass = clear_password + &salt;
        let mut hasher = Sha3_256::default();
        hasher.input(pass.as_bytes());
        let out = hasher.result();
        let computed_hash = encode(&out[..]);
        computed_hash + "." + &salt
    }

    #[derive(Default)]
    pub struct MockConnection {
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
    pub struct MockConnectionManager;

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
    pub struct MockError {}

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

    pub const MOCK_REPO_FACTORY: ReposFactoryMock = ReposFactoryMock {};
    pub const MOCK_USERS: UsersRepoMock = UsersRepoMock {};
    pub const MOCK_IDENT: IdentitiesRepoMock = IdentitiesRepoMock {};
    pub static MOCK_EMAIL: &'static str = "example@mail.com";
    pub static MOCK_PASSWORD: &'static str = "password";
    pub static MOCK_TOKEN: &'static str = "token";
    pub static MOCK_SAGA_ID: &'static str = "saga_id";
    pub static GOOGLE_TOKEN: &'static str =
        "ya29.GlxRBXyOU1dfRmFEdVE1oOK3SyQ6UKh4RTESu0J-C19N2o5RCQVEALMi5DKlgctjTQclLCrLQkUovOb05ikfYQdZ2paFja9Uf4GN1hoysgp_dDr9NLgvfo7fGth \
         Y8A";
    pub static FACEBOOK_TOKEN: &'static str =
        "AQDr-FG4bmYyrhYGk9ZJg1liqTRBfKfRbXopSd72_Qjexg3e4ybh9EJZFErHwyhw0oKyUOEbCQSalC4D8b3B2r4eJiyEmyW-E_ESsVnyThn27j8KEDDfsxCwUJxZY6fD \
         wZt9LWMEHnHYEnFxABIupKN8y8bj_SH8wxIZoDm-YzZtYbj7VUf9g0vPKOkA_1hnjjW8TGrEKmbhFZLWLj6wJgC3uek3D3MahUhd_k3K-4BjOJNyXa8h_ESPQWNHt9sII \
         IDmhAw5X4iVmdbte7tQWf6y96vd_muwA4hKMRxzc7gMQo16tcI7hazQaJ1rJj39G8poG9Ac7AjdO6O7vSnYB9IqeLFbhKH56IyJoCR_05e2tg";

}
