//! Users Services, presents CRUD operations with users

use chrono::Utc;
use std::time::SystemTime;

use base64::encode;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;
use futures::future;
use futures::Future;
use r2d2::ManageConnection;
use uuid::Uuid;

use stq_static_resources::{Provider, TokenType};
use stq_types::UserId;

use super::types::ServiceFuture;
use super::util::{password_create, password_verify};
use errors::Error;
use models::ResetToken;
use models::{ChangeIdentityPassword, NewIdentity, UpdateIdentity};
use models::{EmailVerifyApplyToken, NewUser, ResetApplyToken, UpdateUser, User, UsersSearchTerms};
use repos::repo_factory::ReposFactory;
use services::jwt::JWTService;
use services::Service;

pub trait UsersService {
    /// Returns user by ID
    fn get(&self, user_id: UserId) -> ServiceFuture<Option<User>>;
    /// Returns total user count
    fn count(&self, only_active_users: bool) -> ServiceFuture<i64>;
    /// Returns current user
    fn current(&self) -> ServiceFuture<Option<User>>;
    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: UserId, count: i64) -> ServiceFuture<Vec<User>>;
    /// Deactivates specific user
    fn deactivate(&self, user_id: UserId) -> ServiceFuture<User>;
    /// Deletes user by saga id
    fn delete_by_saga_id(&self, saga_id: String) -> ServiceFuture<User>;
    /// Creates new user
    fn create(&self, payload: NewIdentity, user_payload: Option<NewUser>) -> ServiceFuture<User>;
    /// Get email verification token
    fn get_email_verification_token(&self, email: String) -> ServiceFuture<String>;
    /// Verifies email
    fn verify_email(&self, token_arg: String) -> ServiceFuture<EmailVerifyApplyToken>;
    /// Updates specific user
    fn update(&self, user_id: UserId, payload: UpdateUser) -> ServiceFuture<User>;
    /// Change user password
    fn change_password(&self, payload: ChangeIdentityPassword) -> ServiceFuture<bool>;
    /// Get password reset token
    fn get_password_reset_token(&self, email_arg: String) -> ServiceFuture<String>;
    /// Apply password reset
    fn password_reset_apply(&self, token: String, new_pass: String) -> ServiceFuture<ResetApplyToken>;
    /// Creates reset token
    fn reset_token_create() -> String;
    /// Find by email
    fn find_by_email(&self, email: String) -> ServiceFuture<Option<User>>;
    /// Search users limited by `from`, `skip` and `count` parameters
    fn search(&self, from: Option<UserId>, skip: i64, count: i64, term: UsersSearchTerms) -> ServiceFuture<Vec<User>>;
    /// Set block status for specific user
    fn set_block_status(&self, user_id: UserId, is_blocked: bool) -> ServiceFuture<User>;
    /// Fuzzy search users by email
    fn fuzzy_search_by_email(&self, term_email: String) -> ServiceFuture<Vec<User>>;
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > UsersService for Service<T, M, F>
{
    /// Returns user by ID
    fn get(&self, user_id: UserId) -> ServiceFuture<Option<User>> {
        let current_uid = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!("Getting user {}", user_id);

        self.spawn_on_pool(move |conn| {
            let users_repo = repo_factory.create_users_repo(&conn, current_uid);
            users_repo
                .find(user_id)
                .map_err(|e: FailureError| e.context("Service users, get endpoint error occured.").into())
        })
    }

    /// Returns total user count
    fn count(&self, only_active_users: bool) -> ServiceFuture<i64> {
        let current_uid = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!("Getting user count");

        self.spawn_on_pool(move |conn| {
            let users_repo = repo_factory.create_users_repo(&conn, current_uid);
            users_repo
                .count(only_active_users)
                .map_err(|e: FailureError| e.context("Service `users`, `count` endpoint error occurred.").into())
        })
    }

    /// Returns current user
    fn current(&self) -> ServiceFuture<Option<User>> {
        if let Some(id) = self.dynamic_context.user_id {
            let repo_factory = self.static_context.repo_factory.clone();

            debug!("Fetching current user ({})", id);

            self.spawn_on_pool(move |conn| {
                let users_repo = repo_factory.create_users_repo(&conn, Some(id));
                users_repo
                    .find(id)
                    .map_err(|e: FailureError| e.context("Service users, current endpoint error occured.").into())
            })
        } else {
            Box::new(future::ok(None))
        }
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: UserId, count: i64) -> ServiceFuture<Vec<User>> {
        let current_uid = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!("Fetching {} users starting from {}", count, from);

        self.spawn_on_pool(move |conn| {
            let users_repo = repo_factory.create_users_repo(&conn, current_uid);
            users_repo
                .list(from, count)
                .map_err(|e: FailureError| e.context("Service users, list endpoint error occured.").into())
        })
    }

    /// Deactivates specific user
    fn deactivate(&self, user_id: UserId) -> ServiceFuture<User> {
        let current_uid = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!("Deactivating user {}", &user_id);

        self.spawn_on_pool(move |conn| {
            let users_repo = repo_factory.create_users_repo(&conn, current_uid);
            users_repo
                .deactivate(user_id)
                .map_err(|e: FailureError| e.context("Service users, deactivate endpoint error occured.").into())
        })
    }

    /// Set block status for specific user
    fn set_block_status(&self, user_id: UserId, is_blocked: bool) -> ServiceFuture<User> {
        let current_uid = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();
        debug!("Set block status {} for user {}", is_blocked, &user_id);

        self.spawn_on_pool(move |conn| {
            let users_repo = repo_factory.create_users_repo(&conn, current_uid);
            users_repo
                .set_block_status(user_id, is_blocked)
                .map_err(|e: FailureError| e.context("Service users, set_block_status endpoint error occured.").into())
        })
    }

    /// Deactivates specific user
    fn delete_by_saga_id(&self, saga_id: String) -> ServiceFuture<User> {
        let current_uid = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!("Deleting user with saga ID {}", &saga_id);

        self.spawn_on_pool(move |conn| {
            let users_repo = repo_factory.create_users_repo(&conn, current_uid);
            users_repo
                .delete_by_saga_id(saga_id)
                .map_err(|e: FailureError| e.context("Service users, delete_by_saga_id endpoint error occured.").into())
        })
    }

    /// Creates new user
    fn create(&self, payload: NewIdentity, user_payload: Option<NewUser>) -> ServiceFuture<User> {
        let current_uid = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!(
            "Creating new user with payload: {:?} and user_payload: {:?}",
            &payload, &user_payload
        );

        self.spawn_on_pool(move |conn| {
            let users_repo = repo_factory.create_users_repo(&conn, current_uid);
            let ident_repo = repo_factory.create_identities_repo(&conn);

            conn.transaction::<User, FailureError, _>(move || {
                ident_repo
                    .email_exists(payload.email.to_string())
                    .and_then(|exists| {
                        if !exists {
                            let new_user = user_payload.unwrap_or(NewUser::from(payload.clone()));
                            users_repo.create(new_user)
                        } else {
                            Err(Error::Validate(validation_errors!({"email": ["email" => "Email already exists"]})).into())
                        }
                    }).and_then(move |user| {
                        ident_repo
                            .create(
                                payload.email,
                                payload.password.map(password_create),
                                payload.provider,
                                user.id,
                                payload.saga_id,
                            ).map(|_| user)
                    })
            }).map_err(|e: FailureError| e.context("Service users, create endpoint error occured.").into())
        })
    }

    /// Get verification token
    fn get_email_verification_token(&self, email: String) -> ServiceFuture<String> {
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let reset_repo = repo_factory.create_reset_token_repo(&conn);

            let _res = reset_repo.delete_by_email(email.clone(), TokenType::EmailVerify);

            let new_token = Self::reset_token_create();
            let reset_token = ResetToken {
                token: new_token,
                email: email.clone(),
                token_type: TokenType::EmailVerify,
                created_at: SystemTime::now(),
            };

            reset_repo
                .create(reset_token)
                .map(|t| t.token)
                .map_err(|e| e.context("Can not create reset token").into())
                .map_err(|e: FailureError| e.context("Service users, resend_verification_link endpoint error occured.").into())
        })
    }

    /// Verifies email
    fn verify_email(&self, token_arg: String) -> ServiceFuture<EmailVerifyApplyToken> {
        let repo_factory = self.static_context.repo_factory.clone();
        let secret = self.static_context.jwt_private_key.clone();
        let service = self.clone();

        let fut = self
            .spawn_on_pool(move |conn| {
                {
                    let users_repo = repo_factory.create_users_repo_with_sys_acl(&conn);
                    let reset_repo = repo_factory.create_reset_token_repo(&conn);

                    let reset_token: ResetToken = reset_repo
                        .find_by_token(token_arg.clone(), TokenType::EmailVerify)
                        .map_err(|e| e.context(Error::InvalidToken))?;

                    let reset_token = reset_repo
                        .delete_by_token(reset_token.token.clone(), TokenType::EmailVerify)
                        .map_err(|e| e.context("Unable to delete token"))?;

                    let user = match SystemTime::now().duration_since(reset_token.created_at) {
                        Ok(elapsed) => {
                            if elapsed.as_secs() < 3600 {
                                let user = users_repo.find_by_email(reset_token.email.clone())?;

                                if let Some(user) = user {
                                    let update = UpdateUser {
                                        email_verified: Some(true),
                                        ..Default::default()
                                    };

                                    users_repo.update(user.id.clone(), update)
                                } else {
                                    Err(Error::InvalidToken
                                        .context(format!("User with email {} not found!", reset_token.email))
                                        .into())
                                }
                            } else {
                                Err(Error::InvalidToken.into())
                            }
                        }
                        Err(_) => Err(Error::InvalidToken.into()),
                    }?;

                    Ok(user)
                }.map_err(|e: FailureError| e.context("Service users, verify_email endpoint error occured.").into())
            }).and_then(move |user| {
                let provider = Provider::Email;
                let exp = Utc::now().timestamp() + 120; // TODO: change now() + expire_config_value
                service
                    .create_jwt(user.id, exp, secret, provider)
                    .and_then(move |token| future::ok(EmailVerifyApplyToken { token, user }))
            });

        Box::new(fut)
    }

    /// Updates specific user
    fn update(&self, user_id: UserId, payload: UpdateUser) -> ServiceFuture<User> {
        let current_uid = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!("Updating user {} with payload: {:?}", &user_id, &payload);

        self.spawn_on_pool(move |conn| {
            let users_repo = repo_factory.create_users_repo(&conn, current_uid);
            users_repo
                .find(user_id.clone())
                .and_then(move |_user| users_repo.update(user_id, payload))
                .map_err(|e: FailureError| e.context("Service users, update endpoint error occured.").into())
        })
    }

    fn change_password(&self, payload: ChangeIdentityPassword) -> ServiceFuture<bool> {
        match self.dynamic_context.user_id {
            Some(current_uid) => {
                let repo_factory = self.static_context.repo_factory.clone();

                debug!("Updating user password {}", &current_uid);

                self.spawn_on_pool(move |conn| {
                    let ident_repo = repo_factory.create_identities_repo(&conn);
                    let old_password = payload.old_password.clone();
                    let new_password = payload.new_password.clone();

                    conn.transaction::<bool, FailureError, _>(move || {
                        ident_repo
                            .find_by_id_provider(current_uid.clone(), Provider::Email)
                            .and_then(move |identity| {
                                let ident_clone = identity.clone();
                                if let Some(passwd) = ident_clone.password {
                                    let verified = password_verify(&passwd, old_password);

                                    match verified {
                                        Ok(verified) => Ok((verified, identity)),
                                        Err(e) => Err(e),
                                    }
                                } else {
                                    error!("No password in db for user with Email provider, user_id: {}", &ident_clone.user_id);
                                    Err(Error::Validate(validation_errors!({"password": ["password" => "Wrong password"]})).into())
                                }
                            }).and_then(move |(verified, identity)| {
                                if !verified {
                                    //password not verified
                                    Err(Error::Validate(validation_errors!({"password": ["password" => "Wrong password"]})).into())
                                } else {
                                    //password verified
                                    debug!("Changing password for identity {:?}", &identity);
                                    let update = UpdateIdentity {
                                        password: Some(password_create(new_password)),
                                    };

                                    ident_repo.update(identity, update).map(|_| true)
                                }
                            })
                    }).map_err(|e: FailureError| e.context("Service users, change_password endpoint error occured.").into())
                })
            }
            None => Box::new(future::err(
                Error::Forbidden.context("Only authorized user can change password").into(),
            )),
        }
    }

    fn get_password_reset_token(&self, email_arg: String) -> ServiceFuture<String> {
        let email = email_arg.clone();
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let reset_repo = repo_factory.create_reset_token_repo(&conn);
            let ident_repo = repo_factory.create_identities_repo(&conn);
            let users_repo = repo_factory.create_users_repo_with_sys_acl(&conn);
            users_repo
                .find_by_email(email.clone())
                .and_then(|user| {
                    user.ok_or_else(|| Error::Validate(validation_errors!({"email": ["email" => "Email does not exists"]})).into())
                        .and_then(|user| {
                            if !user.email_verified {
                                //email not verified
                                Err(Error::Validate(validation_errors!({"email": ["email" => "Email not verified"]})).into())
                            } else {
                                Ok(user)
                            }
                        })
                }).and_then(|_| {
                    ident_repo
                        .find_by_email_provider(email.clone(), Provider::Email)
                        .map_err(|e| e.context("Identity by email search failure").context(Error::InvalidToken).into())
                }).and_then(|ident| {
                    debug!("Found identity {:?}, generating reset token.", &ident);

                    debug!("Removing previous tokens for {} if any", &ident.email);
                    let _res = reset_repo.delete_by_email(ident.email.clone(), TokenType::PasswordReset);

                    debug!("Generating new token for {}", &ident.email);
                    let new_token = Self::reset_token_create();
                    let reset_token = ResetToken {
                        token: new_token,
                        email: ident.email.clone(),
                        token_type: TokenType::PasswordReset,
                        created_at: SystemTime::now(),
                    };

                    reset_repo
                        .create(reset_token)
                        .map_err(|e| e.context("Cannot create reset token").into())
                }).map(|t| t.token)
                .map_err(|e: FailureError| e.context("Service users, password_reset_request endpoint error occured.").into())
        })
    }

    fn password_reset_apply(&self, token_arg: String, new_pass: String) -> ServiceFuture<ResetApplyToken> {
        let repo_factory = self.static_context.repo_factory.clone();
        let secret = self.static_context.jwt_private_key.clone();
        let service = self.clone();

        debug!("Resetting password for token {}.", &token_arg);

        let fut = self
            .spawn_on_pool(move |conn| {
                {
                    let reset_repo = repo_factory.create_reset_token_repo(&conn);
                    let ident_repo = repo_factory.create_identities_repo(&conn);

                    let reset_token = reset_repo
                        .find_by_token(token_arg.clone(), TokenType::PasswordReset)
                        .map_err(|e| e.context("Reset token by token search failure").context(Error::InvalidToken))?;

                    let reset_token = reset_repo
                        .delete_by_token(reset_token.token.clone(), TokenType::PasswordReset)
                        .map_err(|e| e.context("Unable to delete token"))?;

                    debug!("Checking reset token's {:?} expiration", &reset_token);
                    let identity = match SystemTime::now().duration_since(reset_token.created_at) {
                        Ok(elapsed) => {
                            if elapsed.as_secs() < 3600 {
                                let ident = ident_repo.find_by_email_provider(reset_token.email.clone(), Provider::Email)?;
                                debug!("Token check successful, resetting password for identity {:?}", &ident);
                                let update = UpdateIdentity {
                                    password: Some(password_create(new_pass)),
                                };

                                ident_repo.update(ident, update)
                            } else {
                                Err(Error::InvalidToken.context(format!("Token {:?} has expired", &reset_token)).into())
                            }
                        }
                        Err(_) => Err(Error::InvalidToken.into()),
                    }?;

                    Ok(identity)
                }.map_err(|e: FailureError| e.context("Service users, password_reset_apply endpoint error occured.").into())
            }).and_then(move |identity| {
                let exp = Utc::now().timestamp() + 120; // TODO: change now() + expire_config_value
                service
                    .create_jwt(identity.user_id, exp, secret, identity.provider)
                    .and_then(move |token| {
                        Ok(ResetApplyToken {
                            token,
                            email: identity.email,
                        })
                    })
            });

        Box::new(fut)
    }

    fn reset_token_create() -> String {
        let new_token = Uuid::new_v4().to_string();
        encode(&new_token)
    }

    /// Find by email
    fn find_by_email(&self, email: String) -> ServiceFuture<Option<User>> {
        let current_uid = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!("Getting user by email {}", email);

        self.spawn_on_pool(move |conn| {
            let users_repo = repo_factory.create_users_repo(&conn, current_uid);
            users_repo
                .find_by_email(email)
                .map_err(|e: FailureError| e.context("Service users, find by email endpoint error occured.").into())
        })
    }

    /// Search users limited by `from`, `skip` and `count` parameters
    fn search(&self, from: Option<UserId>, skip: i64, count: i64, term: UsersSearchTerms) -> ServiceFuture<Vec<User>> {
        let current_uid = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!(
            "Searching for users (from: {:?}, skip: {}, count: {}) with payload: {:?}",
            from, skip, count, term
        );

        self.spawn_on_pool(move |conn| {
            let users_repo = repo_factory.create_users_repo(&conn, current_uid);
            users_repo
                .search(from, skip, count, term)
                .map_err(|e: FailureError| e.context("Service `users`, `search` endpoint error occured.").into())
        })
    }

    /// Fuzzy search users by email
    fn fuzzy_search_by_email(&self, term_email: String) -> ServiceFuture<Vec<User>> {
        let current_uid = self.dynamic_context.user_id;
        let repo_factory = self.static_context.repo_factory.clone();

        debug!("Searching for users email containing {}", term_email);

        self.spawn_on_pool(move |conn| {
            let users_repo = repo_factory.create_users_repo(&conn, current_uid);
            users_repo
                .fuzzy_search_by_email(term_email)
                .map_err(|e: FailureError| e.context("Service users, fuzzy_search_by_email endpoint error occured.").into())
        })
    }
}

#[cfg(test)]
pub mod tests {

    use std::sync::Arc;

    use tokio_core::reactor::Core;

    use stq_static_resources::Provider;
    use stq_types::UserId;

    use repos::repo_factory::tests::*;
    use services::users::UsersService;

    #[test]
    fn test_get_user() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(UserId(1)), handle);
        let work = service.get(UserId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().id, UserId(1));
    }

    #[test]
    fn test_current_user() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(UserId(1)), handle);
        let work = service.current();
        let result = core.run(work).unwrap();
        assert_eq!(result.unwrap().email, MOCK_EMAIL.to_string());
    }

    #[test]
    fn test_current_user_without_user_email() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(None, handle);
        let work = service.current();
        let result = core.run(work);
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_list() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(UserId(1)), handle);
        let work = service.list(UserId(1), 5);
        let result = core.run(work).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_create_allready_existed() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(UserId(1)), handle);
        let new_ident = create_new_identity(
            MOCK_EMAIL.to_string(),
            MOCK_PASSWORD.to_string(),
            Provider::Email,
            MOCK_SAGA_ID.to_string(),
        );
        let work = service.create(new_ident, None);
        let result = core.run(work);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_create_user() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(UserId(1)), handle);
        let new_ident = create_new_identity(
            "new_user@mail.com".to_string(),
            MOCK_PASSWORD.to_string(),
            Provider::Email,
            MOCK_SAGA_ID.to_string(),
        );
        let work = service.create(new_ident, None);
        let result = core.run(work).unwrap();
        assert_eq!(result.email, "new_user@mail.com".to_string());
    }

    #[test]
    fn test_update() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(UserId(1)), handle);
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
        let service = create_service(Some(UserId(1)), handle);
        let work = service.deactivate(UserId(1));
        let result = core.run(work).unwrap();
        assert_eq!(result.id, UserId(1));
        assert_eq!(result.is_active, false);
    }
}
