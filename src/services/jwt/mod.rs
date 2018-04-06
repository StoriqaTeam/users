//! Json Web Token Services, presents creating jwt from google, facebook and email + password
pub mod profile;

use std::str;
use std::sync::Arc;

use base64::decode;
use diesel::Connection;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use r2d2::{ManageConnection, Pool};
use futures::future;
use futures::{Future, IntoFuture};
use futures_cpupool::CpuPool;
use hyper::header::{Authorization, Bearer};
use hyper::{Headers, Method};
use jsonwebtoken::{encode, Header};
use serde;
use serde_json;
use sha3::{Digest, Sha3_256};
use stq_http::client::ClientHandle;
use uuid::Uuid;

use self::profile::{Email, FacebookProfile, GoogleProfile, IntoUser};
use config::{Config, JWT as JWTConfig, OAuth};
use models::{self, JWTPayload, NewEmailIdentity, NewIdentity, NewUser, Provider, ProviderOauth, User, UserStatus, JWT};
use services::error::ServiceError;
use services::types::ServiceFuture;
use repos::repo_factory::ReposFactory;

/// JWT services, responsible for JsonWebToken operations
pub trait JWTService {
    /// Creates new JWT token by email
    fn create_token_email(&self, payload: NewEmailIdentity) -> ServiceFuture<JWT>;
    /// Creates new JWT token by google
    fn create_token_google(self, oauth: ProviderOauth) -> ServiceFuture<JWT>;
    /// Creates new JWT token by facebook
    fn create_token_facebook(self, oauth: ProviderOauth) -> ServiceFuture<JWT>;
    /// Verifies password
    fn password_verify(db_hash: String, clear_password: String) -> Result<bool, ServiceError>;
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
}

impl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> JWTServiceImpl<T, M, F>
{
    pub fn new(db_pool: Pool<M>, cpu_pool: CpuPool, http_client: ClientHandle, config: Config, repo_factory: F) -> Self {
        Self {
            db_pool,
            cpu_pool,
            http_client,
            saga_addr: config.saga_addr.url.clone(),
            google_config: config.google,
            facebook_config: config.facebook,
            jwt_config: config.jwt,
            repo_factory,
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

trait ProfileService<T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static, P: Email>
     {
    fn create_token(self, provider: Provider, secret: String, info_url: String, headers: Option<Headers>) -> ServiceFuture<JWT>;

    fn get_profile(&self, url: String, headers: Option<Headers>) -> ServiceFuture<P>;

    fn profile_status(&self, profile: P, provider: Provider) -> ServiceFuture<ProfileStatus>;

    fn create_jwt(&self, id: i32, status: UserStatus, secret: String) -> ServiceFuture<JWT> {
        debug!("Creating token for user {}", id);
        let tokenpayload = JWTPayload::new(id);
        Box::new(
            encode(&Header::default(), &tokenpayload, secret.as_ref())
                .map_err(|_| ServiceError::Parse(format!("Couldn't encode jwt: {:?}.", tokenpayload)))
                .into_future()
                .inspect(move |token| {
                    debug!("Token {} created successfully for id {}", token, id);
                })
                .and_then(move |token| future::ok(JWT { token, status })),
        )
    }

    fn create_profile(&self, profile: P, provider: Provider) -> Result<i32, ServiceError>;

    fn update_profile(&self, conn: &T, profile: P) -> Result<i32, ServiceError>;

    fn get_id(&self, profile: P, provider: Provider) -> ServiceFuture<i32>;
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
        secret: String,
        info_url: String,
        headers: Option<Headers>,
    ) -> Box<Future<Item = JWT, Error = ServiceError>> {
        let service = Arc::new(self);
        let db_pool = service.db_pool.clone();
        let thread_pool = service.cpu_pool.clone();
        let future = service
            .get_profile(info_url, headers)
            .and_then({
                let provider = provider.clone();
                let s = service.clone();
                move |profile| {
                    let profile_clone = profile.clone();
                    s.profile_status(profile, provider)
                        .map(|status| (status, profile_clone))
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
                move |(id, status)| s.create_jwt(id, status, secret)
            });

        Box::new(future)
    }

    fn get_profile(&self, url: String, headers: Option<Headers>) -> ServiceFuture<P> {
        Box::new(
            self.http_client
                .request::<P>(Method::Get, url, None, headers)
                .map_err(|e| {
                    ServiceError::HttpClient(format!(
                        "Failed to receive user info from provider. {}",
                        e.to_string()
                    ))
                }),
        )
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

        let created_user = self.http_client
            .request::<User>(Method::Post, url, Some(body), None)
            .wait()?;

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
                    users_repo
                        .update(user.id, update_user)
                        .map_err(ServiceError::from)
                        .map(|u| u.id.0)
                }
            })
    }

    fn profile_status(&self, profile: P, provider: Provider) -> ServiceFuture<ProfileStatus> {
        Box::new({
            let db_pool = self.db_pool.clone();
            let repo_factory = self.repo_factory.clone();
            self.cpu_pool.spawn_fn(move || {
                db_pool
                    .get()
                    .map_err(|e| ServiceError::Connection(e.into()))
                    .and_then(move |conn| {
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

    fn get_id(&self, profile: P, provider: Provider) -> ServiceFuture<i32> {
        Box::new({
            let db_pool = self.db_pool.clone();
            let repo_factory = self.repo_factory.clone();
            self.cpu_pool.spawn_fn(move || {
                db_pool
                    .get()
                    .map_err(|e| ServiceError::Connection(e.into()))
                    .and_then(move |conn| {
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
    fn create_token_email(&self, payload: NewEmailIdentity) -> ServiceFuture<JWT> {
        let r2d2_clone = self.db_pool.clone();
        let jwt_secret_key = self.jwt_config.secret_key.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            r2d2_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let ident_repo = repo_factory.create_identities_repo(&conn);

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
                                        let new_ident_clone = new_ident.clone();
                                        ident_repo
                                            .find_by_email_provider(new_ident.email.clone(), Provider::Email)
                                            .map_err(ServiceError::from)
                                            .and_then(move |identity| {
                                                Self::password_verify(
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
                                }
                            })
                            .and_then(move |id| {
                                let tokenpayload = JWTPayload::new(id);
                                encode(&Header::default(), &tokenpayload, jwt_secret_key.as_ref())
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
    fn create_token_google(self, oauth: ProviderOauth) -> ServiceFuture<JWT> {
        let url = self.google_config.info_url.clone();
        let mut headers = Headers::new();
        headers.set(Authorization(Bearer { token: oauth.token }));
        let jwt_secret_key = self.jwt_config.secret_key.clone();
        <JWTServiceImpl<T, M, F> as ProfileService<T, GoogleProfile>>::create_token(
            self,
            Provider::Google,
            jwt_secret_key,
            url,
            Some(headers),
        )
    }

    /// https://developers.facebook.com/docs/facebook-login/manually-build-a-login-flow
    /// Creates new JWT token by facebook
    fn create_token_facebook(self, oauth: ProviderOauth) -> ServiceFuture<JWT> {
        let info_url = self.facebook_config.info_url.clone();
        let url = format!(
            "{}?fields=first_name,last_name,gender,email,name&access_token={}",
            info_url, oauth.token
        );
        let jwt_secret_key = self.jwt_config.secret_key.clone();
        <JWTServiceImpl<T, M, F> as ProfileService<T, FacebookProfile>>::create_token(
            self,
            Provider::Facebook,
            jwt_secret_key,
            url,
            None,
        )
    }

    fn password_verify(db_hash: String, clear_password: String) -> Result<bool, ServiceError> {
        let v: Vec<&str> = db_hash.split('.').collect();
        if v.len() != 2 {
            Err(ServiceError::Validate(
                validation_errors!({"password": ["password" => "Password in db has wrong format"]}),
            ))
        } else {
            let salt = v[1];
            let pass = clear_password + salt;
            let mut hasher = Sha3_256::default();
            hasher.input(pass.as_bytes());
            let out = hasher.result();
            let computed_hash = decode(v[0])
                .map_err(|_| ServiceError::Validate(validation_errors!({"password": ["password" => "Password in db has wrong format"]})))?;
            Ok(computed_hash == &out[..])
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;
    use tokio_core::reactor::Core;

    use repos::repo_factory::tests::*;

    use services::jwt::JWTService;
    use models::*;

    #[test]
    fn test_jwt_email() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_jwt_service(handle);
        let new_user = create_new_email_identity(MOCK_EMAIL.to_string(), MOCK_PASSWORD.to_string());
        let work = service.create_token_email(new_user);
        let result = core.run(work).unwrap();
        assert_eq!(
            result.token,
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxfQ.u29q4XLsMSDxPJngHHQV4THkbx-Tn9g7HjcLPEKMT1U"
        );
    }

    #[test]
    fn test_jwt_email_not_found() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_jwt_service(handle);
        let new_user = create_new_email_identity("not found email".to_string(), MOCK_PASSWORD.to_string());
        let work = service.create_token_email(new_user);
        let result = core.run(work);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_jwt_password_incorrect() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_jwt_service(handle);
        let new_user = create_new_email_identity(MOCK_EMAIL.to_string(), "wrong password".to_string());
        let work = service.create_token_email(new_user);
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
        let work = service.create_token_google(oauth);
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
        let work = service.create_token_facebook(oauth);
        let result = core.run(work).unwrap();
        assert_eq!(result.token, "token");
    }
}
