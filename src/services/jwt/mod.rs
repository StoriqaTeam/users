//! Json Web Token Services, presents creating jwt from google, facebook and email + password
pub mod profile;

use std::sync::Arc;

use diesel::Connection;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use futures::future;
use futures::{Future, IntoFuture};
use futures_cpupool::CpuPool;
use hyper::header::{Authorization, Bearer};
use hyper::{Headers, Method};
use jsonwebtoken::{encode, Algorithm, Header};
use r2d2::{ManageConnection, Pool};
use serde;
use serde_json;
use stq_http::client::ClientHandle;
use uuid::Uuid;

use self::profile::{Email, FacebookProfile, GoogleProfile, IntoUser};
use super::util::password_verify;
use config::{Config, JWT as JWTConfig, OAuth};
use models::{self, JWTPayload, NewEmailIdentity, NewIdentity, NewUser, Provider, ProviderOauth, User, UserStatus, JWT};
use repos::repo_factory::ReposFactory;
use services::error::ServiceError;
use services::types::ServiceFuture;

/// JWT services, responsible for JsonWebToken operations
pub trait JWTService {
    /// Creates new JWT token by email
    fn create_token_email(&self, payload: NewEmailIdentity, exp: i64) -> ServiceFuture<JWT>;
    /// Creates new JWT token by google
    fn create_token_google(self, oauth: ProviderOauth, exp: i64) -> ServiceFuture<JWT>;
    /// Creates new JWT token by facebook
    fn create_token_facebook(self, oauth: ProviderOauth, exp: i64) -> ServiceFuture<JWT>;
}

/// JWT services, responsible for JsonWebToken operations
#[derive(Clone)]
pub struct JWTServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub http_client: ClientHandle,
    pub saga_addr: String,
    pub google_config: OAuth,
    pub facebook_config: OAuth,
    pub jwt_config: JWTConfig,
    pub repo_factory: F,
    pub jwt_private_key: Vec<u8>,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > JWTServiceImpl<T, M, F>
{
    pub fn new(
        db_pool: Pool<M>,
        cpu_pool: CpuPool,
        http_client: ClientHandle,
        config: Config,
        repo_factory: F,
        jwt_private_key: Vec<u8>,
    ) -> Self {
        Self {
            db_pool,
            cpu_pool,
            http_client,
            saga_addr: config.saga_addr.url.clone(),
            google_config: config.google,
            facebook_config: config.facebook,
            jwt_config: config.jwt,
            repo_factory,
            jwt_private_key,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProfileStatus {
    // New user, new identity
    NewUser,
    // User exists with other identities
    NewIdentity,
    // User and identity for this email exist
    ExistingProfile,
}

/// Profile service trait, presents standard scheme for receiving profile information from providers

trait ProfileService<T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static, P: Email> {
    fn create_token(self, provider: Provider, secret: Vec<u8>, info_url: String, headers: Option<Headers>, exp: i64) -> ServiceFuture<JWT>;

    fn get_profile(&self, url: String, headers: Option<Headers>) -> ServiceFuture<P>;

    fn profile_status(&self, profile: P, provider: Provider) -> ServiceFuture<ProfileStatus>;

    fn create_profile(&self, profile: P, provider: Provider) -> Result<i32, ServiceError>;

    fn update_profile(&self, conn: &T, profile: P) -> Result<i32, ServiceError>;

    fn get_id(&self, profile: P, provider: Provider) -> ServiceFuture<i32>;

    fn create_jwt(&self, id: i32, exp: i64, status: UserStatus, secret: Vec<u8>, provider: Provider) -> ServiceFuture<JWT> {
        debug!("Creating token for user {}, at {}", id, exp);
        let tokenpayload = JWTPayload::new(id, exp, provider);
        Box::new(
            encode(&Header::new(Algorithm::RS256), &tokenpayload, secret.as_ref())
                .map_err(|_| ServiceError::Parse(format!("Couldn't encode jwt: {:?}.", tokenpayload)))
                .into_future()
                .inspect(move |token| {
                    debug!("Token {} created successfully for id {}", token, id);
                })
                .and_then(move |token| future::ok(JWT { token, status })),
        )
    }
}

impl<
        P,
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T> + 'static,
        F: ReposFactory<T> + 'static,
    > ProfileService<T, P> for JWTServiceImpl<T, M, F>
where
    P: Email + Clone + Send + 'static,
    NewUser: From<P>,
    P: for<'a> serde::Deserialize<'a>,
    P: IntoUser,
{
    fn create_token(
        self,
        provider: Provider,
        secret: Vec<u8>,
        info_url: String,
        headers: Option<Headers>,
        exp: i64,
    ) -> Box<Future<Item = JWT, Error = ServiceError>> {
        let service = Arc::new(self);
        let db_pool = service.db_pool.clone();
        let thread_pool = service.cpu_pool.clone();
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
                move |(status, profile)| -> ServiceFuture<(i32, UserStatus)> {
                    Box::new({
                        thread_pool.spawn_fn(move || {
                            db_pool
                                .get()
                                .map_err(|e| ServiceError::Connection(e.into()))
                                .and_then(move |conn| match status {
                                    ProfileStatus::ExistingProfile => {
                                        debug!("User exists for this profile. Looking up ID.");
                                        s.get_id(profile, provider)
                                            .inspect(move |id| debug!("Fetched user ID: {}", &id))
                                            .map(|id| (id, UserStatus::Exists))
                                            .wait()
                                    }
                                    ProfileStatus::NewUser => {
                                        debug!("No user matches profile. Creating one");
                                        s.create_profile(profile.clone(), provider).map(|id| {
                                            debug!("Created user {} for profile.", &id);
                                            (id, UserStatus::New(id))
                                        })
                                    }
                                    ProfileStatus::NewIdentity => {
                                        debug!("User exists, tying new identity to them.");
                                        s.update_profile(&conn, profile).map(|id| {
                                            debug!("Created identity for user {}", id);
                                            (id, UserStatus::New(id))
                                        })
                                    }
                                })
                        })
                    })
                }
            })
            .and_then({
                let s = service.clone();
                move |(id, status)| s.create_jwt(id, exp, status, secret, provider_clone)
            });

        Box::new(future)
    }

    fn get_profile(&self, url: String, headers: Option<Headers>) -> ServiceFuture<P> {
        Box::new(
            self.http_client
                .request::<serde_json::Value>(Method::Get, url, None, headers)
                .map_err(|e| ServiceError::HttpClient(format!("Failed to receive user info from provider. {}", e.to_string())))
                .and_then(|val| match val["email"].is_null() {
                    true => Err(ServiceError::Validate(
                        validation_errors!({"email": ["email" => "Email required but not provided"]}),
                    )),
                    false => serde_json::from_value::<P>(val).map_err(ServiceError::from),
                }),
        )
    }

    fn profile_status(&self, profile: P, provider: Provider) -> ServiceFuture<ProfileStatus> {
        Box::new({
            let db_pool = self.db_pool.clone();
            let repo_factory = self.repo_factory.clone();
            self.cpu_pool.spawn_fn(move || {
                db_pool.get().map_err(|e| ServiceError::Connection(e.into())).and_then(move |conn| {
                    let users_repo = repo_factory.create_users_repo_with_sys_acl(&conn);
                    let ident_repo = repo_factory.create_identities_repo(&conn);
                    conn.transaction(move || {
                        users_repo
                            .email_exists(profile.get_email())
                            .and_then(|user_exists| match user_exists {
                                false => Ok(ProfileStatus::NewUser),
                                true => ident_repo
                                    .email_provider_exists(profile.get_email(), provider)
                                    .map(|identity_exists| match identity_exists {
                                        false => ProfileStatus::NewIdentity,
                                        true => ProfileStatus::ExistingProfile,
                                    }),
                            })
                            .map_err(ServiceError::from)
                    })
                })
            })
        })
    }

    fn create_profile(&self, profile_arg: P, provider: Provider) -> Result<i32, ServiceError> {
        let new_user = NewUser::from(profile_arg.clone());

        let url = format!("{}/{}", &self.saga_addr, "create_account");

        let body = serde_json::to_string(&models::SagaCreateProfile {
            user: Some(new_user.clone()),
            identity: NewIdentity {
                email: new_user.email,
                password: None,
                provider,
                saga_id: Uuid::new_v4().to_string(),
            },
        }).map_err(ServiceError::from)?;

        let created_user = self.http_client.request::<User>(Method::Post, url, Some(body), None).wait()?;

        Ok(created_user.id.0)
    }

    fn update_profile(&self, conn: &T, profile: P) -> Result<i32, ServiceError> {
        let mut users_repo = self.repo_factory.create_users_repo_with_sys_acl(conn);
        users_repo
            .find_by_email(profile.get_email())
            .map_err(ServiceError::from)
            .map(|user| (profile, user))
            .and_then(move |(profile, user)| {
                let update_user = profile.merge_into_user(user.clone());

                if update_user.is_empty() {
                    Ok(user.id.0)
                } else {
                    users_repo.update(user.id, update_user).map_err(ServiceError::from).map(|u| u.id.0)
                }
            })
    }

    fn get_id(&self, profile: P, provider: Provider) -> ServiceFuture<i32> {
        Box::new({
            let db_pool = self.db_pool.clone();
            let repo_factory = self.repo_factory.clone();
            self.cpu_pool.spawn_fn(move || {
                db_pool.get().map_err(|e| ServiceError::Connection(e.into())).and_then(move |conn| {
                    let ident_repo = repo_factory.create_identities_repo(&conn);

                    ident_repo
                        .find_by_email_provider(profile.get_email(), provider)
                        .map_err(ServiceError::from)
                        .map(|ident| ident.user_id.0)
                })
            })
        })
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > JWTService for JWTServiceImpl<T, M, F>
{
    /// Creates new JWT token by email
    fn create_token_email(&self, payload: NewEmailIdentity, exp: i64) -> ServiceFuture<JWT> {
        let r2d2_clone = self.db_pool.clone();
        let jwt_private_key = self.jwt_private_key.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            r2d2_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let ident_repo = repo_factory.create_identities_repo(&conn);
                    let mut users_repo = repo_factory.create_users_repo_with_sys_acl(&conn);

                    conn.transaction::<JWT, ServiceError, _>(move || {
                        ident_repo
                            .email_provider_exists(payload.email.to_string(), Provider::Email)
                            .map_err(ServiceError::from)
                            .map(|exists| (exists, payload))
                            .and_then(move |(exists, new_ident)| -> Result<i32, ServiceError> {
                                match exists {
                                    // email does not exist
                                    false => Err(ServiceError::Validate(
                                        validation_errors!({"email": ["email" => "Email not found"]}),
                                    )),
                                    // email exists, checking password
                                    true => {
                                        users_repo
                                            .find_by_email(new_ident.email.clone())
                                            .map_err(ServiceError::from)
                                            .map(|user| (new_ident, user))
                                            .and_then(move |(new_ident, user)| {
                                                match user.email_verified {
                                                    true => {
                                                        let new_ident_clone = new_ident.clone();
                                                        ident_repo
                                                            .find_by_email_provider(new_ident.email.clone(), Provider::Email)
                                                            .map_err(ServiceError::from)
                                                            .and_then(move |identity| {
                                                                password_verify(
                                                                    identity.password.unwrap().clone(),
                                                                    new_ident.password.clone(),
                                                                )
                                                            })
                                                            .map(move |verified| (verified, new_ident_clone))
                                                            .and_then(move |(verified, new_ident)| -> Result<i32, ServiceError> {
                                                                match verified {
                                                                    //password not verified
                                                                    false => Err(ServiceError::Validate(
                                                                        validation_errors!({"password": ["password" => "Wrong password"]}),
                                                                    )),
                                                                    //password verified
                                                                    true => ident_repo
                                                                        .find_by_email_provider(new_ident.email, Provider::Email)
                                                                        .map_err(ServiceError::from)
                                                                        .map(|ident| ident.user_id.0),
                                                                }
                                                            })
                                                    }
                                                    false => Err(ServiceError::Validate(
                                                        validation_errors!({"email": ["email" => "Email not verified"]}),
                                                    )),
                                                }
                                            })
                                    }
                                }
                            })
                            .and_then(move |id| {
                                let tokenpayload = JWTPayload::new(id, exp, Provider::Email);
                                encode(&Header::new(Algorithm::RS256), &tokenpayload, jwt_private_key.as_ref())
                                    .map_err(|_| ServiceError::Parse(format!("Couldn't encode jwt: {:?}", tokenpayload)))
                                    .and_then(|t| {
                                        Ok(JWT {
                                            token: t,
                                            status: UserStatus::Exists,
                                        })
                                    })
                            })
                    })
                })
        }))
    }

    /// https://developers.google.com/identity/protocols/OpenIDConnect#validatinganidtoken
    /// Creates new JWT token by google
    fn create_token_google(self, oauth: ProviderOauth, exp: i64) -> ServiceFuture<JWT> {
        let url = self.google_config.info_url.clone();
        let mut headers = Headers::new();
        headers.set(Authorization(Bearer { token: oauth.token }));
        let jwt_private_key = self.jwt_private_key.clone();
        <JWTServiceImpl<T, M, F> as ProfileService<T, GoogleProfile>>::create_token(
            self,
            Provider::Google,
            jwt_private_key,
            url,
            Some(headers),
            exp,
        )
    }

    /// https://developers.facebook.com/docs/facebook-login/manually-build-a-login-flow
    /// Creates new JWT token by facebook
    fn create_token_facebook(self, oauth: ProviderOauth, exp: i64) -> ServiceFuture<JWT> {
        let info_url = self.facebook_config.info_url.clone();
        let url = format!(
            "{}?fields=first_name,last_name,gender,email,name&access_token={}",
            info_url, oauth.token
        );
        let jwt_private_key = self.jwt_private_key.clone();
        <JWTServiceImpl<T, M, F> as ProfileService<T, FacebookProfile>>::create_token(
            self,
            Provider::Facebook,
            jwt_private_key,
            url,
            None,
            exp,
        )
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;
    use tokio_core::reactor::Core;

    use repos::repo_factory::tests::*;

    use models::*;
    use services::jwt::JWTService;

    #[test]
    fn test_jwt_email() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_jwt_service(handle);
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
        let service = create_jwt_service(handle);
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
        let service = create_jwt_service(handle);
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
        let service = create_jwt_service(handle);
        let oauth = ProviderOauth {
            token: GOOGLE_TOKEN.to_string(),
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
        let service = create_jwt_service(handle);
        let oauth = ProviderOauth {
            token: FACEBOOK_TOKEN.to_string(),
        };
        let exp = 1;
        let work = service.create_token_facebook(oauth, exp);
        let result = core.run(work).unwrap();
        assert_eq!(result.token, "token");
    }
}
