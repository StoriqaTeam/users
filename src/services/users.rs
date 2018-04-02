//! Users Services, presents CRUD operations with users

use std::time::SystemTime;

use base64::encode;
use diesel::Connection;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use futures::Future;
use futures::IntoFuture;
use futures::future;
use futures_cpupool::CpuPool;
use hyper::Method;
use rand;
use rand::Rng;
use serde_json;
use sha3::{Digest, Sha3_256};
use uuid::Uuid;
use r2d2::{ManageConnection, Pool};

use stq_http::client::ClientHandle;

use models::{NewIdentity, Provider, UpdateIdentity};
use models::{NewUser, UpdateUser, User, UserId};
use models::{ResetMail, ResetToken};
use repos::repo_factory::ReposFactory;

use config::Notifications;

use super::error::ServiceError;
use super::types::ServiceFuture;

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
    /// Resends verification link
    fn resend_verification_link(&self, email: String) -> ServiceFuture<bool>;
    /// Verifies email
    fn verify_email(&self, token_arg: String) -> ServiceFuture<bool>;
    /// Updates specific user
    fn update(&self, user_id: UserId, payload: UpdateUser) -> ServiceFuture<User>;
    /// Request password reset
    fn password_reset_request(&self, email_arg: String) -> ServiceFuture<bool>;
    /// Apply password reset
    fn password_reset_apply(&self, email_arg: String, token_arg: String) -> ServiceFuture<bool>;
    /// Creates hashed password
    fn password_create(clear_password: String) -> String;
    /// Send email
    fn send_mail(http_client: ClientHandle, notif_config: Notifications, to: String, subject: String, text: String) -> ServiceFuture<bool>;
}

/// Users services, responsible for User-related CRUD operations
pub struct UsersServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub http_client: ClientHandle,
    user_id: Option<i32>,
    pub notif_conf: Notifications,
    pub repo_factory: F,
}

impl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> UsersServiceImpl<T, M, F>
{
    pub fn new(
        db_pool: Pool<M>,
        cpu_pool: CpuPool,
        http_client: ClientHandle,
        user_id: Option<i32>,
        notif_conf: Notifications,
        repo_factory: F,
    ) -> Self {
        Self {
            db_pool,
            cpu_pool,
            http_client,
            user_id,
            notif_conf,
            repo_factory,
        }
    }
}

impl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> UsersService for UsersServiceImpl<T, M, F>
{
    /// Returns user by ID
    fn get(&self, user_id: UserId) -> ServiceFuture<User> {
        let db_clone = self.db_pool.clone();
        let current_uid = self.user_id.clone();
        let repo_factory = self.repo_factory.clone();

        debug!("Getting user {}", user_id);

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let mut users_repo = repo_factory.create_users_repo(&conn, current_uid);
                    users_repo.find(user_id).map_err(ServiceError::from)
                })
        }))
    }

    /// Returns current user
    fn current(&self) -> ServiceFuture<User> {
        if let Some(id) = self.user_id {
            let db_clone = self.db_pool.clone();
            let current_uid = self.user_id.clone();
            let repo_factory = self.repo_factory.clone();

            debug!("Fetching current user ({})", id);

            Box::new(self.cpu_pool.spawn_fn(move || {
                db_clone
                    .get()
                    .map_err(|e| ServiceError::Connection(e.into()))
                    .and_then(move |conn| {
                        let mut users_repo = repo_factory.create_users_repo(&conn, current_uid);
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
        let current_uid = self.user_id.clone();
        let repo_factory = self.repo_factory.clone();

        debug!("Fetching {} users starting from {}", count, from);

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let mut users_repo = repo_factory.create_users_repo(&conn, current_uid);
                    users_repo.list(from, count).map_err(ServiceError::from)
                })
        }))
    }

    /// Deactivates specific user
    fn deactivate(&self, user_id: UserId) -> ServiceFuture<User> {
        let db_clone = self.db_pool.clone();
        let current_uid = self.user_id.clone();
        let repo_factory = self.repo_factory.clone();

        debug!("Deactivating user {}", &user_id);

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let mut users_repo = repo_factory.create_users_repo(&conn, current_uid);
                    users_repo.deactivate(user_id).map_err(ServiceError::from)
                })
        }))
    }

    /// Deactivates specific user
    fn delete_by_saga_id(&self, saga_id: String) -> ServiceFuture<User> {
        let db_clone = self.db_pool.clone();
        let current_uid = self.user_id.clone();
        let repo_factory = self.repo_factory.clone();

        debug!("Deleting user with saga ID {}", &saga_id);

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let mut users_repo = repo_factory.create_users_repo(&conn, current_uid);
                    users_repo
                        .delete_by_saga_id(saga_id)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Creates new user
    fn create(&self, payload: NewIdentity, user_payload: Option<NewUser>) -> ServiceFuture<User> {
        let db_clone = self.db_pool.clone();
        let current_uid = self.user_id.clone();
        let http_clone = self.http_client.clone();
        let notif_config = self.notif_conf.clone();
        let repo_factory = self.repo_factory.clone();

        debug!(
            "Creating new user with payload: {:?} and user_payload: {:?}",
            &payload, &user_payload
        );

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_clone
                        .get()
                        .map_err(|e| ServiceError::Connection(e.into()))
                        .and_then(move |conn| {
                            let mut users_repo = repo_factory.create_users_repo(&conn, current_uid);
                            let ident_repo = repo_factory.create_identities_repo(&conn);
                            let reset_repo = repo_factory.create_reset_token_repo(&conn);

                            conn.transaction::<(ResetToken, User), ServiceError, _>(move || {
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
                                            .map(|ident| (ident, user))
                                    })
                                    .and_then(move |(ident, user)| {
                                        let new_token = Uuid::new_v4().to_string();
                                        let reset_token = ResetToken {
                                            token: new_token,
                                            email: ident.email.clone(),
                                            created_at: SystemTime::now(),
                                        };

                                        reset_repo.delete_by_email(reset_token.email.clone())?;

                                        reset_repo
                                            .create(reset_token)
                                            .map(|token| (token, user))
                                            .map_err(|_e| ServiceError::Unknown("Cannot create reset token".to_string()))
                                    })
                            })
                        })
                })
                .and_then(move |(token, user)| {
                    let to = token.email.clone();
                    let subject = "Email verification".to_string();
                    let text = format!(
                        "{}/{}",
                        notif_config.verify_email_path.clone(),
                        token.token.clone()
                    );
                    let user_clone = user.clone();

                    debug!("Sending email notification to user");

                    Self::send_mail(http_clone.clone(), notif_config.clone(), to, subject, text)
                        .map(|_| user)
                        .or_else(|_| future::ok(user_clone))
                }),
        )
    }

    /// Resends verification link
    fn resend_verification_link(&self, email: String) -> ServiceFuture<bool> {
        let db_clone = self.db_pool.clone();
        let http_clone = self.http_client.clone();
        let notif_config = self.notif_conf.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_clone
                        .get()
                        .map_err(|e| ServiceError::Connection(e.into()))
                        .and_then(move |conn| {
                            let reset_repo = repo_factory.create_reset_token_repo(&conn);

                            reset_repo
                                .delete_by_email(email.clone())
                                .map_err(|_e| ServiceError::Unknown("Cannot create reset token".to_string()))
                                .and_then(|_token| {
                                    let new_token = Uuid::new_v4().to_string();
                                    let reset_token = ResetToken {
                                        token: new_token,
                                        email: email.clone(),
                                        created_at: SystemTime::now(),
                                    };

                                    reset_repo
                                        .create(reset_token)
                                        .map_err(|_e| ServiceError::Unknown("Cannot create reset token".to_string()))
                                })
                        })
                })
                .and_then(move |token| {
                    let to = token.email.clone();
                    let subject = "Email verification".to_string();
                    let text = format!(
                        "{}/{}",
                        notif_config.verify_email_path.clone(),
                        token.token.clone()
                    );

                    Self::send_mail(http_clone.clone(), notif_config.clone(), to, subject, text).or_else(|_| future::ok(true))
                }),
        )
    }

    /// Verifies email
    fn verify_email(&self, token_arg: String) -> ServiceFuture<bool> {
        let db_clone = self.db_pool.clone();
        let http_clone = self.http_client.clone();
        let notif_config = self.notif_conf.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_clone
                        .get()
                        .map_err(|e| ServiceError::Connection(e.into()))
                        .and_then(move |conn| {
                            let mut users_repo = repo_factory.create_users_repo_with_sys_acl(&conn);
                            let reset_repo = repo_factory.create_reset_token_repo(&conn);

                            reset_repo
                                .find_by_token(token_arg.clone())
                                .map_err(|_e| ServiceError::InvalidToken)
                                .and_then(|reset_token| {
                                    reset_repo
                                        .delete_by_token(reset_token.token.clone())
                                        .map_err(|_e| {
                                            println!("Unable to delete token");
                                            ServiceError::Unknown("".to_string())
                                        })
                                })
                                .and_then(
                                    move |reset_token| match SystemTime::now().duration_since(reset_token.created_at) {
                                        Ok(elapsed) => {
                                            if elapsed.as_secs() < 3600 {
                                                users_repo
                                                    .find_by_email(reset_token.email.clone())
                                                    .and_then(|user| {
                                                        let update = UpdateUser {
                                                            phone: None,
                                                            first_name: None,
                                                            last_name: None,
                                                            middle_name: None,
                                                            gender: None,
                                                            birthdate: None,
                                                            is_active: None,
                                                            email_verified: Some(true),
                                                        };

                                                        users_repo.update(user.id.clone(), update)
                                                    })
                                                    .map_err(|_e| ServiceError::InvalidToken)
                                            } else {
                                                Err(ServiceError::InvalidToken)
                                            }
                                        }
                                        Err(_) => Err(ServiceError::InvalidToken),
                                    },
                                )
                        })
                })
                .and_then(move |user| {
                    let to = user.email.clone();
                    let subject = "Email verification".to_string();
                    let text = "Email for linked account has been verified".to_string();

                    Self::send_mail(http_clone.clone(), notif_config.clone(), to, subject, text).or_else(|_| future::ok(true))
                }),
        )
    }

    /// Updates specific user
    fn update(&self, user_id: UserId, payload: UpdateUser) -> ServiceFuture<User> {
        let db_clone = self.db_pool.clone();
        let current_uid = self.user_id.clone();
        let repo_factory = self.repo_factory.clone();

        debug!("Updating user {} with payload: {:?}", &user_id, &payload);

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let mut users_repo = repo_factory.create_users_repo(&conn, current_uid);
                    users_repo
                        .find(user_id.clone())
                        .and_then(move |_user| users_repo.update(user_id, payload))
                        .map_err(ServiceError::from)
                })
        }))
    }

    fn password_reset_request(&self, email_arg: String) -> ServiceFuture<bool> {
        let db_clone = self.db_pool.clone();
        let email = email_arg.clone();
        let http_clone = self.http_client.clone();
        let notif_config = self.notif_conf.clone();
        let repo_factory = self.repo_factory.clone();

        debug!("Resetting password for email {}", &email_arg);

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_clone
                        .get()
                        .map_err(|e| ServiceError::Connection(e.into()))
                        .and_then(move |conn| {
                            let reset_repo = repo_factory.create_reset_token_repo(&conn);
                            let ident_repo = repo_factory.create_identities_repo(&conn);

                            ident_repo
                                .find_by_email_provider(email.clone(), Provider::Email)
                                .map_err(|_e| ServiceError::InvalidToken)
                                .and_then(|ident| {
                                    debug!("Found identity {:?}, generating reset token.", &ident);
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
                        })
                })
                .and_then(move |token| {
                    let to = token.email.clone();
                    let subject = "Password reset".to_string();
                    let text = format!(
                        "{}/{}",
                        notif_config.reset_password_path.clone(),
                        token.token.clone()
                    );

                    Self::send_mail(http_clone.clone(), notif_config.clone(), to, subject, text).or_else(|_| future::ok(true))
                }),
        )
    }

    fn password_reset_apply(&self, token_arg: String, new_pass: String) -> ServiceFuture<bool> {
        let db_clone = self.db_pool.clone();
        let http_clone = self.http_client.clone();
        let notif_config = self.notif_conf.clone();
        let repo_factory = self.repo_factory.clone();

        debug!("Resetting password for token {}.", &token_arg);

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_clone
                        .get()
                        .map_err(|e| ServiceError::Connection(e.into()))
                        .and_then(move |conn| {
                            let reset_repo = repo_factory.create_reset_token_repo(&conn);
                            let ident_repo = repo_factory.create_identities_repo(&conn);

                            reset_repo
                                .find_by_token(token_arg.clone())
                                .map_err(|_e| ServiceError::InvalidToken)
                                .and_then(|reset_token| {
                                    reset_repo
                                        .delete_by_token(reset_token.token.clone())
                                        .map_err(|_e| {
                                            println!("Unable to delete token");
                                            ServiceError::Unknown("".to_string())
                                        })
                                })
                                .and_then(move |reset_token| {
                                    debug!("Checking reset token's {:?} expiration", &reset_token);
                                    match SystemTime::now().duration_since(reset_token.created_at) {
                                        Ok(elapsed) => {
                                            if elapsed.as_secs() < 3600 {
                                                ident_repo
                                                    .find_by_email_provider(reset_token.email.clone(), Provider::Email)
                                                    .and_then(move |ident| {
                                                        debug!(
                                                            "Token check successful, resetting password for identity {:?}",
                                                            &ident
                                                        );
                                                        let update = UpdateIdentity {
                                                            password: Some(Self::password_create(new_pass)),
                                                        };

                                                        ident_repo.update(ident, update)
                                                    })
                                                    .map_err(|_e| ServiceError::InvalidToken)
                                            } else {
                                                error!("Token {:?} has expired", &reset_token);
                                                Err(ServiceError::InvalidToken)
                                            }
                                        }
                                        Err(_) => Err(ServiceError::InvalidToken),
                                    }
                                })
                        })
                })
                .and_then(move |ident| {
                    let to = ident.email.clone();
                    let subject = "Password reset success".to_string();
                    let text = "Password for linked account has been successfully reset.".to_string();

                    Self::send_mail(http_clone.clone(), notif_config.clone(), to, subject, text).or_else(|_| future::ok(true))
                }),
        )
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

    fn send_mail(http_client: ClientHandle, notif_config: Notifications, to: String, subject: String, text: String) -> ServiceFuture<bool> {
        let url = format!("{}/sendmail", notif_config.url.clone());

        Box::new(
            serde_json::to_string(&ResetMail { to, subject, text })
                .map_err(ServiceError::from)
                .into_future()
                .and_then(move |body| {
                    http_client
                        .request::<String>(Method::Post, url, Some(body), None)
                        .map(|_v| true)
                        .map_err(|_e| ServiceError::Connection(format_err!("Error sending email")))
                }),
        )
    }
}

#[cfg(test)]
pub mod tests {

    use std::sync::Arc;
    use tokio_core::reactor::Core;

    use repos::repo_factory::tests::*;

    use services::users::UsersService;
    use models::*;

    #[test]
    fn test_get_user() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_users_service(Some(1), handle);
        let work = service.get(UserId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.id, UserId(1));
    }

    #[test]
    fn test_current_user() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_users_service(Some(1), handle);
        let work = service.current();
        let result = core.run(work).unwrap();
        assert_eq!(result.email, MOCK_EMAIL.to_string());
    }

    #[test]
    fn test_current_user_without_user_email() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_users_service(None, handle);
        let work = service.current();
        let result = core.run(work);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_list() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_users_service(Some(1), handle);
        let work = service.list(1, 5);
        let result = core.run(work).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_create_allready_existed() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_users_service(Some(1), handle);
        let new_ident = create_new_identity(MOCK_EMAIL.to_string(), MOCK_PASSWORD.to_string(), Provider::Email, MOCK_SAGA_ID.to_string());
        let work = service.create(new_ident, None);
        let result = core.run(work);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_create_user() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_users_service(Some(1), handle);
        let new_ident = create_new_identity("new_user@mail.com".to_string(), MOCK_PASSWORD.to_string(), Provider::Email, MOCK_SAGA_ID.to_string());
        let work = service.create(new_ident, None);
        let result = core.run(work).unwrap();
        assert_eq!(result.email, "new_user@mail.com".to_string());
    }

    #[test]
    fn test_update() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_users_service(Some(1), handle);
        let new_user = create_update_user(MOCK_EMAIL.to_string());
        let work = service.update(UserId(1), new_user);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, UserId(1));
        assert_eq!(result.email, MOCK_EMAIL.to_string());
    }

    #[test]
    fn test_deactivate() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_users_service(Some(1), handle);
        let work = service.deactivate(UserId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.id, UserId(1));
        assert_eq!(result.is_active, false);
    }
}
