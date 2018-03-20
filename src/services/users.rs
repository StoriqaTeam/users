//! Users Services, presents CRUD operations with users

use std::time::SystemTime;

use futures::future;
use futures::Future;
use futures_cpupool::CpuPool;
use hyper::Method;
use sha3::{Digest, Sha3_256};
use rand;
use rand::Rng;
use base64::encode;
use diesel::Connection;
use uuid::Uuid;
use serde_json;

use stq_acl::{UnauthorizedACL};
use http::client::ClientHandle;

use models::{NewUser, UpdateUser, User, UserId};
use models::{NewIdentity, UpdateIdentity, Provider};
use models::{ResetToken, ResetMail};
use repos::identities::{IdentitiesRepo, IdentitiesRepoImpl};
use repos::users::{UsersRepo, UsersRepoImpl};
use repos::reset_token::{ResetTokenRepo, ResetTokenRepoImpl};
use repos::types::DbPool;
use repos::acl::{ApplicationAcl, BoxedAcl, RolesCacheImpl};

use config::Notifications;

use super::types::ServiceFuture;
use super::error::ServiceError;

pub trait UsersService {
    /// Returns user by ID
    fn get(&self, user_id: UserId) -> ServiceFuture<User>;
    /// Returns current user
    fn current(&self) -> ServiceFuture<User>;
    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<User>>;
    /// Deactivates specific user
    fn deactivate(&self, user_id: UserId) -> ServiceFuture<User>;
    /// Deletes user by saga id
    fn delete_by_saga_id(&self, saga_id: String) -> ServiceFuture<User>;
    /// Creates new user
    fn create(&self, payload: NewIdentity, user_payload: Option<NewUser>) -> ServiceFuture<User>;
    /// Updates specific user
    fn update(&self, user_id: UserId, payload: UpdateUser) -> ServiceFuture<User>;
    /// creates hashed password
    fn password_create(clear_password: String) -> String;
    /// Request password reset
    fn password_reset_request(&self, email_arg: String) -> ServiceFuture<bool>;
    /// Apply password reset
    fn password_reset_apply(&self, email_arg: String, token_arg: String) -> ServiceFuture<bool>;
}

/// Users services, responsible for User-related CRUD operations
pub struct UsersServiceImpl {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub http_client: ClientHandle,
    roles_cache: RolesCacheImpl,
    user_id: Option<i32>,
    pub notif_conf: Notifications,
}

impl UsersServiceImpl {
    pub fn new(
        db_pool: DbPool,
        cpu_pool: CpuPool,
        http_client: ClientHandle,
        roles_cache: RolesCacheImpl,
        user_id: Option<i32>,
        notif_conf: Notifications,
    ) -> Self {
        Self {
            db_pool,
            cpu_pool,
            http_client,
            roles_cache,
            user_id,
            notif_conf,
        }
    }
}

fn acl_for_id(roles_cache: RolesCacheImpl, user_id: Option<i32>) -> BoxedAcl {
    user_id.map_or(Box::new(UnauthorizedACL::default()) as BoxedAcl, |id| {
        (Box::new(ApplicationAcl::new(roles_cache, id)) as BoxedAcl)
    })
}

impl UsersService for UsersServiceImpl {
    /// Returns user by ID
    fn get(&self, user_id: UserId) -> ServiceFuture<User> {
        let db_clone = self.db_pool.clone();
        let roles_cache = self.roles_cache.clone();
        let current_uid = self.user_id.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_clone
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
            let db_clone = self.db_pool.clone();
            let roles_cache = self.roles_cache.clone();
            let current_uid = self.user_id.clone();

            Box::new(self.cpu_pool.spawn_fn(move || {
                db_clone
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
        let db_clone = self.db_pool.clone();
        let roles_cache = self.roles_cache.clone();
        let current_uid = self.user_id.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_clone
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
        let db_clone = self.db_pool.clone();
        let roles_cache = self.roles_cache.clone();
        let current_uid = self.user_id.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache.clone(), current_uid);
                    let mut users_repo = UsersRepoImpl::new(&conn, acl);
                    users_repo.deactivate(user_id).map_err(ServiceError::from)
                })
        }))
    }

    /// Deactivates specific user
    fn delete_by_saga_id(&self, saga_id: String) -> ServiceFuture<User> {
        let db_clone = self.db_pool.clone();
        let roles_cache = self.roles_cache.clone();
        let current_uid = self.user_id.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache.clone(), current_uid);
                    let mut users_repo = UsersRepoImpl::new(&conn, acl);
                    users_repo.delete_by_saga_id(saga_id).map_err(ServiceError::from)
                })
        }))
    }

    /// Creates new user
    fn create(&self, payload: NewIdentity, user_payload: Option<NewUser>) -> ServiceFuture<User> {
        let db_clone = self.db_pool.clone();
        let roles_cache = self.roles_cache.clone();
        let current_uid = self.user_id.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_clone
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
                                let new_user;
                                match user_payload {
                                    Some(usr) => new_user = usr.clone(),
                                    None => new_user = NewUser::from(new_ident.clone()),
                                }
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
                                        new_ident.saga_id,
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
        let db_clone = self.db_pool.clone();
        let roles_cache = self.roles_cache.clone();
        let current_uid = self.user_id.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_clone
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


    fn password_reset_request(&self, email_arg: String) -> ServiceFuture<bool> {
        let db_clone = self.db_pool.clone();
        let http_clone = self.http_client.clone();
        let email = email_arg.clone();
        let notif_config = self.notif_conf.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let reset_repo = ResetTokenRepoImpl::new(&conn);
                    let ident_repo = IdentitiesRepoImpl::new(&conn);

                    ident_repo
                        .find_by_email_provider(email.clone(), Provider::Email)
                        .map_err(|_e| ServiceError::InvalidToken)
                        .and_then(|ident| {
                            let new_token = Uuid::new_v4().to_string();
                            let reset_token = ResetToken {
                                token: new_token,
                                email: ident.email.clone(),
                                created_at: SystemTime::now(),
                            };

                            reset_repo
                                .create(reset_token)
                                .map_err(|_e| ServiceError::Unknown("Cannot create reset token".to_string()))
                        })
                        .and_then(|token| {
                            let url = format!(
                                "{}/{}",
                                notif_config.url.clone(),
                                notif_config.sendmail_path.clone()
                            );

                            let link_text = format!(
                                "{}/{}",
                                notif_config.link_path.clone(),
                                token.token.clone(),
                            );

                            http_clone
                                .request::<String>(
                                    Method::Post,
                                    url,
                                    Some(
                                        serde_json::to_string(&ResetMail {
                                            to: token.email.clone(),
                                            subject: "Password reset".to_string(),
                                            text: link_text,
                                        }).unwrap(),
                                    ),
                                    None,
                                )
                                .wait()
                                .map(|_v| true)
                                .map_err(|_e| ServiceError::Connection(
                                    format_err!("Error sending email")
                                ))
                        })
                })
        }))
    }

    fn password_reset_apply(&self, token_arg: String, new_pass: String) -> ServiceFuture<bool> {
        let db_clone = self.db_pool.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let reset_repo = ResetTokenRepoImpl::new(&conn);
                    let ident_repo = IdentitiesRepoImpl::new(&conn);

                    reset_repo
                        .find_by_token(token_arg.clone())
                        .map_err(|_e| ServiceError::InvalidToken)
                        .and_then(|reset_token| {
                            reset_repo
                                .delete(reset_token.token.clone())
                                .map_err(|_e| {
                                    println!("Unable to delete token");
                                    ServiceError::Unknown("".to_string())
                                })
                        })
                        .and_then(move |reset_token| {

                            match SystemTime::now().duration_since(reset_token.created_at) {
                                Ok(elapsed) => {
                                    if elapsed.as_secs() < 3600 {
                                        ident_repo
                                            .find_by_email_provider(reset_token.email.clone(), Provider::Email)
                                            .and_then(move |ident| {
                                                let update = UpdateIdentity {
                                                    password: Some(Self::password_create(new_pass)),
                                                };

                                                ident_repo
                                                    .update(ident, update)
                                                    .map(|_ident| true)
                                            })
                                            .map_err(|_e| ServiceError::InvalidToken)
                                    } else {
                                        Err(ServiceError::InvalidToken)
                                    }
                                },
                                Err(_) => Err(ServiceError::InvalidToken)
                            }
                        })
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
