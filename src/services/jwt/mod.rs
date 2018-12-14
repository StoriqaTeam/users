//! Json Web Token Services, presents creating jwt from google, facebook and email + password
pub mod profile;

use std::sync::Arc;

use chrono::Utc;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;
use futures::future;
use futures::{Future, IntoFuture};
use hyper::header::{Authorization, Bearer};
use hyper::{Headers, Method};
use jsonwebtoken::{encode, Algorithm, Header};
use r2d2::ManageConnection;
use serde;
use serde_json;
use uuid::Uuid;

use stq_http::client::HttpClient;
use stq_static_resources::Provider;
use stq_types::UserId;

use self::profile::{Email, FacebookProfile, GoogleProfile, IntoUser, ProfileStatus};
use super::util::password_verify;
use errors::Error;
use models::jwt::NewUserAdditionalData;
use models::{self, EmailIdentity, JWTPayload, NewIdentity, NewUser, ProviderOauth, User, UserStatus, JWT};
use repos::repo_factory::ReposFactory;
use repos::types::RepoResult;
use services::types::ServiceFuture;
use services::Service;

/// JWT services, responsible for JsonWebToken operations
pub trait JWTService {
    /// Creates new JWT token by email
    fn create_token_email(&self, payload: EmailIdentity, exp: i64) -> ServiceFuture<JWT>;
    /// Creates new JWT token by google
    fn create_token_google(self, oauth: ProviderOauth, exp: i64) -> ServiceFuture<JWT>;
    /// Creates new JWT token by facebook
    fn create_token_facebook(self, oauth: ProviderOauth, exp: i64) -> ServiceFuture<JWT>;
    /// Crates new JWT token
    fn create_jwt(&self, id: UserId, exp: i64, secret: Vec<u8>, provider: Provider) -> ServiceFuture<String> {
        debug!("Creating token for user_id {:?}, at {}", id, exp);
        let tokenpayload = JWTPayload::new(id, exp, provider);
        Box::new(
            encode(&Header::new(Algorithm::RS256), &tokenpayload, secret.as_ref())
                .map_err(|e| {
                    format_err!("{}", e)
                        .context(Error::Parse)
                        .context(format!("Couldn't encode jwt: {:?}.", tokenpayload))
                        .into()
                })
                .into_future()
                .map(move |token| {
                    debug!("Token {} created successfully for user_id {:?}", token, id);
                    token
                }),
        )
    }
    fn refresh_token(&self, old_payload: JWTPayload) -> ServiceFuture<String>;
}

/// Profile service trait, presents standard scheme for receiving profile information from providers

trait ProfileService<T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static, P: Email> {
    fn create_token(
        self,
        provider: Provider,
        info_url: String,
        headers: Option<Headers>,
        additional_data: Option<NewUserAdditionalData>,
        exp: i64,
    ) -> ServiceFuture<JWT>;

    fn get_profile(&self, url: String, headers: Option<Headers>) -> ServiceFuture<P>;

    fn profile_status(&self, profile: P, provider: Provider) -> ServiceFuture<ProfileStatus>;

    fn create_profile(&self, profile: P, provider: Provider, additional_data: Option<NewUserAdditionalData>) -> RepoResult<UserId>;

    fn update_profile(&self, conn: &T, profile: P) -> RepoResult<UserId>;

    fn get_id(&self, profile: P, provider: Provider) -> ServiceFuture<UserId>;
}

impl<
        P,
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T> + 'static,
        F: ReposFactory<T> + 'static,
    > ProfileService<T, P> for Service<T, M, F>
where
    P: Email + Clone + Send + 'static,
    NewUser: From<P>,
    P: for<'a> serde::Deserialize<'a>,
    P: IntoUser,
{
    fn create_token(
        self,
        provider: Provider,
        info_url: String,
        headers: Option<Headers>,
        additional_data: Option<NewUserAdditionalData>,
        exp: i64,
    ) -> ServiceFuture<JWT> {
        let secret = self.static_context.jwt_private_key.clone();
        let service = Arc::new(self);
        let provider_clone = provider.clone();

        let future = service
            .get_profile(info_url, headers)
            .and_then({
                let provider = provider.clone();
                let s = service.clone();
                move |profile| {
                    let profile_clone = profile.clone();
                    s.profile_status(profile, provider).map(|status| (status, profile_clone))
                }
            })
            .and_then({
                let s = service.clone();
                move |(status, profile)| -> ServiceFuture<(UserId, UserStatus)> {
                    s.spawn_on_pool({
                        let s = s.clone();
                        move |conn| match status {
                            ProfileStatus::ExistingProfile => {
                                debug!("User exists for this profile. Looking up ID.");
                                s.get_id(profile, provider)
                                    .inspect(move |id| debug!("Fetched user ID: {}", &id))
                                    .map(|id| (id, UserStatus::Exists))
                                    .wait()
                            }
                            ProfileStatus::NewUser => {
                                debug!("No user matches profile. Creating one");
                                s.create_profile(profile.clone(), provider, additional_data).map(|id| {
                                    debug!("Created user {} for profile.", &id);
                                    (id, UserStatus::New(id))
                                })
                            }
                            ProfileStatus::NewIdentity => {
                                debug!("User exists, trying new identity to them.");
                                s.update_profile(&conn, profile).map(|id| {
                                    debug!("Created identity for user {}", id);
                                    (id, UserStatus::New(id))
                                })
                            }
                        }
                    })
                }
            })
            .and_then({
                let s = service.clone();
                move |(id, status)| {
                    s.create_jwt(id, exp, secret, provider_clone)
                        .and_then(move |token| future::ok(JWT { token, status }))
                }
            })
            .map_err(|e: FailureError| e.context("Service jwt, create_token endpoint error occured.").into());

        Box::new(future)
    }

    fn get_profile(&self, url: String, headers: Option<Headers>) -> ServiceFuture<P> {
        Box::new(
            self.dynamic_context
                .http_client
                .request_json::<serde_json::Value>(Method::Get, url, None, headers)
                .map_err(|e| {
                    e.context("Failed to receive user info from provider. {}")
                        .context(Error::Forbidden)
                        .into()
                })
                .and_then(|val| {
                    if val["email"].is_null() {
                        Err(Error::Validate(validation_errors!({"email": ["email" => "Email required but not provided"]})).into())
                    } else {
                        serde_json::from_value::<P>(val.clone()).map_err(|e| e.context(format!("Can not parse profile: {}", val)).into())
                    }
                })
                .map_err(|e: FailureError| e.context("Service jwt, get_profile endpoint error occured.").into()),
        )
    }

    fn profile_status(&self, profile: P, provider: Provider) -> ServiceFuture<ProfileStatus> {
        let repo_factory = self.static_context.repo_factory.clone();
        self.spawn_on_pool(move |conn| {
            let users_repo = repo_factory.create_users_repo_with_sys_acl(&conn);
            let ident_repo = repo_factory.create_identities_repo(&conn);
            conn.transaction(move || {
                users_repo.email_exists(profile.get_email()).and_then(|user_exists| {
                    if user_exists {
                        ident_repo
                            .email_provider_exists(profile.get_email(), provider)
                            .map(|identity_exists| {
                                if identity_exists {
                                    ProfileStatus::ExistingProfile
                                } else {
                                    ProfileStatus::NewIdentity
                                }
                            })
                    } else {
                        Ok(ProfileStatus::NewUser)
                    }
                })
            })
            .map_err(|e: FailureError| e.context("Service jwt, profile_status endpoint error occured.").into())
        })
    }

    fn create_profile(&self, profile_arg: P, provider: Provider, additional_data: Option<NewUserAdditionalData>) -> RepoResult<UserId> {
        let new_user = NewUser::from(profile_arg.clone());
        let saga_addr = self.static_context.config.saga_addr.url.clone();

        let url = format!("{}/{}", saga_addr, "create_account");

        let additional_data = additional_data.unwrap_or_default();

        serde_json::to_string(&models::SagaCreateProfile {
            user: Some(NewUser {
                referal: additional_data.referal,
                utm_marks: additional_data.utm_marks,
                referer: additional_data.referer,
                country: additional_data.country,
                ..new_user.clone()
            }),
            identity: NewIdentity {
                email: new_user.email,
                password: None,
                provider,
                saga_id: Uuid::new_v4().to_string(),
            },
        })
        .map_err(From::from)
        .and_then(|body| {
            self.dynamic_context
                .http_client
                .request_json::<User>(Method::Post, url, Some(body), None)
                .wait()
                .map_err(|e| e.context(Error::HttpClient).into())
        })
        .map(|created_user| created_user.id)
        .map_err(|e: FailureError| e.context("Service jwt, create_profile saga request failed.").into())
    }

    fn update_profile(&self, conn: &T, profile: P) -> RepoResult<UserId> {
        let users_repo = self.static_context.repo_factory.create_users_repo_with_sys_acl(conn);
        users_repo
            .find_by_email(profile.get_email())
            .and_then(move |user| {
                if let Some(user) = user {
                    if user.is_blocked {
                        error!("User {} is blocked.", user.id);
                        return Err(Error::Validate(validation_errors!({"email": ["email" => "Email is blocked"]})).into());
                    }

                    let update_user = profile.merge_into_user(user.clone());

                    if update_user.is_empty() {
                        Ok(user.id)
                    } else {
                        users_repo.update(user.id, update_user).map(|u| u.id)
                    }
                } else {
                    Err(Error::NotFound
                        .context(format!("User with email {} not found!", profile.get_email()))
                        .into())
                }
            })
            .map_err(|e: FailureError| e.context("Service jwt, update_profile endpoint error occured.").into())
    }

    fn get_id(&self, profile: P, provider: Provider) -> ServiceFuture<UserId> {
        let repo_factory = self.static_context.repo_factory.clone();
        self.spawn_on_pool(move |conn| {
            let ident_repo = repo_factory.create_identities_repo(&conn);

            ident_repo
                .find_by_email_provider(profile.get_email(), provider)
                .map(|ident| ident.user_id)
                .map_err(|e: FailureError| e.context("Service jwt, get_id endpoint error occured.").into())
        })
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > JWTService for Service<T, M, F>
{
    /// Creates new JWT token by email
    fn create_token_email(&self, payload: EmailIdentity, exp: i64) -> ServiceFuture<JWT> {
        let jwt_private_key = self.static_context.jwt_private_key.clone();
        let repo_factory = self.static_context.repo_factory.clone();

        self.spawn_on_pool(move |conn| {
            let ident_repo = repo_factory.create_identities_repo(&conn);
            let users_repo = repo_factory.create_users_repo_with_sys_acl(&conn);

            conn.transaction::<JWT, FailureError, _>(move || {
                ident_repo
                    .email_exists(payload.email.clone())
                    .and_then(move |exists| -> RepoResult<UserId> {
                        if !exists {
                            // email does not exist
                            Err(Error::Validate(validation_errors!({"email": ["email" => "Email not found"]})).into())
                        } else {
                            // email exists, checking password
                            users_repo.find_by_email(payload.email.clone()).and_then(move |user| {
                                if let Some(user) = user {
                                    if user.is_blocked {
                                        error!("User {} is blocked.", user.id);
                                        Err(Error::Validate(validation_errors!({"email": ["email" => "Email is blocked"]})).into())
                                    } else if user.email_verified {
                                        ident_repo
                                            .get_by_email(payload.email.clone())
                                            .and_then(|identity| match identity.provider {
                                                Provider::Email => {
                                                    if let Some(passwd) = identity.password {
                                                        password_verify(&passwd, payload.password.clone())
                                                    } else {
                                                        error!(
                                                            "No password in db for user with Email provider, user_id: {}",
                                                            &identity.user_id
                                                        );
                                                        Err(Error::Validate(
                                                            validation_errors!({"password": ["password" => "Wrong password"]}),
                                                        )
                                                        .into())
                                                    }
                                                }
                                                _ => {
                                                    error!("No password in db for user with email, user_id: {}", &identity.user_id);
                                                    Err(Error::Validate(validation_errors!({"password": ["password" => "Wrong password"]}))
                                                        .into())
                                                }
                                            })
                                            .and_then(move |verified| -> Result<UserId, FailureError> {
                                                if !verified {
                                                    //password not verified
                                                    Err(Error::Validate(validation_errors!({"password": ["password" => "Wrong password"]}))
                                                        .into())
                                                } else {
                                                    //password verified
                                                    ident_repo
                                                        .find_by_email_provider(payload.email, Provider::Email)
                                                        .map(|ident| ident.user_id)
                                                }
                                            })
                                    } else {
                                        Err(Error::Validate(validation_errors!({"email": ["not_verified" => "Email not verified"]})).into())
                                    }
                                } else {
                                    Err(Error::NotFound
                                        .context(format!("User with email {} not found!", payload.email))
                                        .into())
                                }
                            })
                        }
                    })
                    .and_then(move |id| {
                        let tokenpayload = JWTPayload::new(id, exp, Provider::Email);
                        encode(&Header::new(Algorithm::RS256), &tokenpayload, jwt_private_key.as_ref())
                            .map_err(|e| {
                                format_err!("{}", e)
                                    .context(Error::Parse)
                                    .context(format!("Couldn't encode jwt: {:?}.", tokenpayload))
                                    .into()
                            })
                            .and_then(|t| {
                                Ok(JWT {
                                    token: t,
                                    status: UserStatus::Exists,
                                })
                            })
                    })
            })
            .map_err(|e: FailureError| e.context("Service jwt, create_token_email endpoint error occured.").into())
        })
    }

    /// https://developers.google.com/identity/protocols/OpenIDConnect#validatinganidtoken
    /// Creates new JWT token by google
    fn create_token_google(self, oauth: ProviderOauth, exp: i64) -> ServiceFuture<JWT> {
        let url = self.static_context.config.google.info_url.clone();
        let mut headers = Headers::new();
        headers.set(Authorization(Bearer { token: oauth.token }));
        let additional_data = oauth.additional_data;
        <Service<T, M, F> as ProfileService<T, GoogleProfile>>::create_token(
            self,
            Provider::Google,
            url,
            Some(headers),
            additional_data,
            exp,
        )
    }

    /// https://developers.facebook.com/docs/facebook-login/manually-build-a-login-flow
    /// Creates new JWT token by facebook
    fn create_token_facebook(self, oauth: ProviderOauth, exp: i64) -> ServiceFuture<JWT> {
        let info_url = self.static_context.config.facebook.info_url.clone();
        let url = format!(
            "{}?fields=first_name,last_name,gender,email,name&access_token={}",
            info_url, oauth.token
        );
        let additional_data = oauth.additional_data;
        <Service<T, M, F> as ProfileService<T, FacebookProfile>>::create_token(self, Provider::Facebook, url, None, additional_data, exp)
    }

    fn refresh_token(&self, old_payload: JWTPayload) -> ServiceFuture<String> {
        let refresh_timeout = self.static_context.config.tokens.refresh_timeout_s;
        let jwt_expiration_s = self.static_context.config.tokens.jwt_expiration_s;
        let secret = self.static_context.jwt_private_key.clone();

        if old_payload.exp + (refresh_timeout as i64) < Utc::now().timestamp() {
            Box::new(Err(Error::InvalidToken.context("Couldn't refresh jwt, it is expired.").into()).into_future())
        } else {
            let exp = Utc::now().timestamp() + jwt_expiration_s as i64;
            let tokenpayload = JWTPayload::new(old_payload.user_id, exp, old_payload.provider);
            Box::new(
                encode(&Header::new(Algorithm::RS256), &tokenpayload, secret.as_ref())
                    .map_err(|e| {
                        format_err!("{}", e)
                            .context(Error::Parse)
                            .context(format!("Couldn't encode jwt: {:?}.", tokenpayload))
                            .into()
                    })
                    .into_future()
                    .map(move |token| {
                        debug!("Token {} created successfully for user_id {:?}", token, old_payload.user_id);
                        token
                    }),
            )
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use tokio_core::reactor::Core;

    use stq_types::UserId;

    use models::*;
    use repos::repo_factory::tests::*;
    use services::jwt::JWTService;

    #[test]
    fn test_jwt_email() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(UserId(1)), handle);
        let new_user = create_new_email_identity(MOCK_EMAIL.to_string(), MOCK_PASSWORD.to_string());
        let exp = 1;
        let work = service.create_token_email(new_user, exp);
        let result = core.run(work).unwrap();
        assert_eq!(
            result.token,
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJleHAiOjEsInByb3ZpZGVyIjoiRW1haWwifQ.IeWgAVZRzFK1L4JbUkiC42TnTa95OF_Gzdy5PAMwbQJifK9NC_qtxrk9W4S62kYsQaxHLupq2rWhMh4WovHH351EAwgqP7eswsBmEML81jeFuUGQ3Vhlkm9b1x-2H5JJI8lRLkPBcqvJDwUM-_7Jz2Q4qY8vE2SgJ7CcnYFFYpjNELrr1Fm0HJN1hnUhXumY3O8V1W7dm5IfASGZx5uu103EKJsZ9KFwWiSs1ZAzII8jvpL1D2uI4Kq5ESXCve1QRqlfzaAlRbpJEsBENxI7oPV8Bp2FH_qhvhSM957lCNM3GcdgNn3B2Gr3b-T7FUjlZieJbIoels1OScO-Q4vdBg"
        );
    }

    #[test]
    fn test_jwt_email_not_found() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(UserId(1)), handle);
        let new_user = create_new_email_identity("not found email".to_string(), MOCK_PASSWORD.to_string());
        let exp = 1;
        let work = service.create_token_email(new_user, exp);
        let result = core.run(work);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_jwt_password_incorrect() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(UserId(1)), handle);
        let new_user = create_new_email_identity(MOCK_EMAIL.to_string(), "wrong password".to_string());
        let exp = 1;
        let work = service.create_token_email(new_user, exp);
        let result = core.run(work);
        assert_eq!(result.is_err(), true);
    }

    // this test is ignored because of expired access code from google
    #[test]
    #[ignore]
    fn test_jwt_google() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(UserId(1)), handle);
        let oauth = ProviderOauth {
            token: GOOGLE_TOKEN.to_string(),
            additional_data: None,
        };
        let exp = 1;
        let work = service.create_token_google(oauth, exp);
        let result = core.run(work).unwrap();
        assert_eq!(result.token, "token");
    }

    // this test is ignored because of expired access code from google
    #[test]
    #[ignore]
    fn test_jwt_facebook() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_service(Some(UserId(1)), handle);
        let oauth = ProviderOauth {
            token: FACEBOOK_TOKEN.to_string(),
            additional_data: None,
        };
        let exp = 1;
        let work = service.create_token_facebook(oauth, exp);
        let result = core.run(work).unwrap();
        assert_eq!(result.token, "token");
    }
}
