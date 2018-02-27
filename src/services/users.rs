//! Users Services, presents CRUD operations with users

use futures::future;
use futures_cpupool::CpuPool;
use sha3::{Digest, Sha3_256};
use rand;
use rand::Rng;
use base64::encode;
use diesel::Connection;

use stq_acl::{Acl, UnauthorizedACL};

use models::{NewUser, UpdateUser, User, UserId};
use models::{NewIdentity, Provider};
use models::authorization::*;
use repos::identities::{IdentitiesRepo, IdentitiesRepoImpl};
use repos::users::{UsersRepo, UsersRepoImpl};

use super::types::ServiceFuture;
use super::error::ServiceError;
use repos::error::RepoError;
use repos::types::DbPool;
use repos::acl::{ApplicationAcl, BoxedAcl, RolesCacheImpl};

pub trait UsersService {
    /// Returns user by ID
    fn get(&self, user_id: UserId) -> ServiceFuture<User>;
    /// Returns current user
    fn current(&self) -> ServiceFuture<User>;
    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<User>>;
    /// Deactivates specific user
    fn deactivate(&self, user_id: UserId) -> ServiceFuture<User>;
    /// Creates new user
    fn create(&self, payload: NewIdentity) -> ServiceFuture<User>;
    /// Updates specific user
    fn update(&self, user_id: UserId, payload: UpdateUser) -> ServiceFuture<User>;
    /// creates hashed password
    fn password_create(clear_password: String) -> String;
}

/// Users services, responsible for User-related CRUD operations
pub struct UsersServiceImpl {
    pub r2d2_pool: DbPool,
    pub cpu_pool: CpuPool,
    roles_cache: RolesCacheImpl,
    user_id: Option<i32>,
}

impl UsersServiceImpl {
    pub fn new(db_pool: DbPool, cpu_pool: CpuPool, roles_cache: RolesCacheImpl, user_id: Option<i32>) -> Self {
        Self {
            r2d2_pool: db_pool,
            cpu_pool: cpu_pool,
            roles_cache: roles_cache,
            user_id: user_id,
        }
    }
}

fn acl_for_id(roles_cache: RolesCacheImpl, user_id: Option<i32>) -> BoxedAcl {
    user_id.map_or((Box::new(UnauthorizedACL::default()) as BoxedAcl), |id| {
        (Box::new(ApplicationAcl::new(roles_cache, id)) as BoxedAcl)
    })
}

impl UsersService for UsersServiceImpl {
    /// Returns user by ID
    fn get(&self, user_id: UserId) -> ServiceFuture<User> {
        let r2d2_clone = self.r2d2_pool.clone();
        let roles_cache = self.roles_cache.clone();
        let current_uid = self.user_id.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            r2d2_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache.clone(), current_uid);
                    let mut users_repo = UsersRepoImpl::new(&conn, acl);
                    users_repo.find(user_id).map_err(ServiceError::from)
                })
        }))
    }

    /// Returns current user
    fn current(&self) -> ServiceFuture<User> {
        if let Some(id) = self.user_id {
            let r2d2_clone = self.r2d2_pool.clone();
            let roles_cache = self.roles_cache.clone();
            let current_uid = self.user_id.clone();

            Box::new(self.cpu_pool.spawn_fn(move || {
                r2d2_clone
                    .get()
                    .map_err(|e| ServiceError::Connection(e.into()))
                    .and_then(move |conn| {
                        let acl = acl_for_id(roles_cache.clone(), current_uid);
                        let mut users_repo = UsersRepoImpl::new(&conn, acl);
                        users_repo.find(UserId(id)).map_err(ServiceError::from)
                    })
            }))
        } else {
            Box::new(future::err(ServiceError::Unknown(format!(
                "There is no user id in request header."
            ))))
        }
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<User>> {
        let r2d2_clone = self.r2d2_pool.clone();
        let roles_cache = self.roles_cache.clone();
        let current_uid = self.user_id.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            r2d2_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache.clone(), current_uid);
                    let mut users_repo = UsersRepoImpl::new(&conn, acl);
                    users_repo.list(from, count).map_err(ServiceError::from)
                })
        }))
    }

    /// Deactivates specific user
    fn deactivate(&self, user_id: UserId) -> ServiceFuture<User> {
        let r2d2_clone = self.r2d2_pool.clone();
        let roles_cache = self.roles_cache.clone();
        let current_uid = self.user_id.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            r2d2_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache.clone(), current_uid);
                    let mut users_repo = UsersRepoImpl::new(&conn, acl);
                    users_repo.deactivate(user_id).map_err(ServiceError::from)
                })
        }))
    }

    /// Creates new user
    fn create(&self, payload: NewIdentity) -> ServiceFuture<User> {
        let r2d2_clone = self.r2d2_pool.clone();
        let roles_cache = self.roles_cache.clone();
        let current_uid = self.user_id.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            r2d2_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache.clone(), current_uid);
                    let mut users_repo = UsersRepoImpl::new(&conn, acl);
                    let ident_repo = IdentitiesRepoImpl::new(&conn);
                    conn.transaction::<User, ServiceError, _>(move || {
                        ident_repo
                            .email_provider_exists(payload.email.to_string(), Provider::Email)
                            .map(move |exists| (payload, exists))
                            .map_err(ServiceError::from)
                            .and_then(|(payload, exists)| match exists {
                                false => Ok(payload),
                                true => Err(ServiceError::Validate(
                                    validation_errors!({"email": ["email" => "Email already exists"]}),
                                )),
                            })
                            .and_then(move |new_ident| {
                                let new_user = NewUser::from(new_ident.clone());
                                users_repo
                                    .create(new_user)
                                    .map_err(ServiceError::from)
                                    .map(|user| (new_ident, user))
                            })
                            .and_then(move |(new_ident, user)| {
                                ident_repo
                                    .create(
                                        new_ident.email,
                                        new_ident
                                            .password
                                            .clone()
                                            .map(|pass| Self::password_create(pass)),
                                        Provider::Email,
                                        user.id.clone(),
                                    )
                                    .map_err(ServiceError::from)
                                    .map(|_| user)
                            })
                    })
                })
        }))
    }

    /// Updates specific user
    fn update(&self, user_id: UserId, payload: UpdateUser) -> ServiceFuture<User> {
        let r2d2_clone = self.r2d2_pool.clone();
        let roles_cache = self.roles_cache.clone();
        let current_uid = self.user_id.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            r2d2_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache.clone(), current_uid);
                    let mut users_repo = UsersRepoImpl::new(&conn, acl);
                    users_repo
                        .find(user_id.clone())
                        .and_then(move |_user| users_repo.update(user_id, payload))
                        .map_err(ServiceError::from)
                })
        }))
    }

    fn password_create(clear_password: String) -> String {
        let salt = rand::thread_rng()
            .gen_ascii_chars()
            .take(10)
            .collect::<String>();
        let pass = clear_password + &salt;
        let mut hasher = Sha3_256::default();
        hasher.input(pass.as_bytes());
        let out = hasher.result();
        let computed_hash = encode(&out[..]);
        computed_hash + "." + &salt
    }
}
