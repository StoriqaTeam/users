//! Json Web Token Services, presents creating jwt from google, facebook and email + password
pub mod profile;

use std::sync::Arc;
use std::str;

use futures::future;
use futures::{Future, IntoFuture};
use futures_cpupool::CpuPool;
use hyper::{Headers, Method};
use hyper::header::{Authorization, Bearer};
use jsonwebtoken::{encode, Header};
use sha3::{Digest, Sha3_256};
use base64::decode;
use serde;
use diesel::Connection;

use models::{JWTPayload, NewIdentity, NewUser, Provider, ProviderOauth, JWT};
use repos::identities::{IdentitiesRepo, IdentitiesRepoImpl};
use repos::users::{UsersRepo, UsersRepoImpl};
use repos::acl::SystemACL;
use config::JWT as JWTConfig;
use config::OAuth;
use config::Config;
use super::types::ServiceFuture;
use super::error::ServiceError;
use repos::types::DbPool;
use self::profile::{Email, FacebookProfile, GoogleProfile, IntoUser};
use http::client::ClientHandle;

/// JWT services, responsible for JsonWebToken operations
pub trait JWTService {
    /// Creates new JWT token by email
    fn create_token_email(&self, payload: NewIdentity) -> ServiceFuture<JWT>;
    /// Creates new JWT token by google
    fn create_token_google(&self, oauth: ProviderOauth) -> ServiceFuture<JWT>;
    /// Creates new JWT token by facebook
    fn create_token_facebook(&self, oauth: ProviderOauth) -> ServiceFuture<JWT>;
    /// Verifies password
    fn password_verify(db_hash: String, clear_password: String) -> Result<bool, ServiceError>;
}

/// JWT services, responsible for JsonWebToken operations
#[derive(Clone)]
pub struct JWTServiceImpl {
    pub r2d2_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub http_client: ClientHandle,
    pub google_config: OAuth,
    pub facebook_config: OAuth,
    pub jwt_config: JWTConfig,
}

impl JWTServiceImpl {
    pub fn new(r2d2_pool: DbPool, cpu_pool: CpuPool, http_client: ClientHandle, config: Config) -> Self {
        Self {
            r2d2_pool: r2d2_pool,
            cpu_pool: cpu_pool,
            http_client: http_client,
            google_config: config.google,
            facebook_config: config.facebook,
            jwt_config: config.jwt,
        }
    }
}

/// Profile service trait, presents standard scheme for receiving profile information from providers
trait ProfileService<P: Email> {
    fn create_token(&self, provider: Provider, secret: String, info_url: String, headers: Option<Headers>) -> ServiceFuture<JWT>;

    fn get_profile(&self, url: String, headers: Option<Headers>) -> ServiceFuture<P>;

    fn email_exists(&self, profile: P, provider: Provider) -> ServiceFuture<bool>;

    fn create_jwt(&self, id: i32, secret: String) -> ServiceFuture<JWT> {
        let tokenpayload = JWTPayload::new(id);
        Box::new(
            encode(&Header::default(), &tokenpayload, secret.as_ref())
                .map_err(|_| ServiceError::Parse(format!("Couldn't encode jwt: {:?}.", tokenpayload)))
                .into_future()
                .and_then(|t| future::ok(JWT { token: t })),
        )
    }

    fn create_profile(
        &self,
        users_repo: UsersRepoImpl,
        ident_repo: IdentitiesRepoImpl,
        profile: P,
        provider: Provider,
    ) -> Result<i32, ServiceError>;

    fn update_profile(&self, users_repo: UsersRepoImpl, profile: P) -> Result<i32, ServiceError>;

    fn create_or_update_profile(&self, profile: P, provider: Provider) -> ServiceFuture<i32>;

    fn get_id(&self, profile: P, provider: Provider) -> ServiceFuture<i32>;
}

impl<P> ProfileService<P> for JWTServiceImpl
where
    P: Email + Clone + Send + 'static,
    NewUser: From<P>,
    P: for<'a> serde::Deserialize<'a>,
    P: IntoUser,
{
    fn create_token(&self, provider: Provider, secret: String, info_url: String, headers: Option<Headers>) -> ServiceFuture<JWT> {
        let service = self.clone();
        let service_clone = self.clone();
        let service_clone2 = self.clone();
        let provider_clone = provider.clone();

        let future = service
            .get_profile(info_url, headers)
            .and_then(move |profile| {
                let profile_clone = profile.clone();
                service
                    .email_exists(profile, provider_clone)
                    .map(|exists| (exists, profile_clone))
            })
            .and_then(move |(exists, profile)| -> ServiceFuture<i32> {
                match exists {
                    // identity email + provider  doesn't exist
                    false => service_clone.create_or_update_profile(profile, provider),
                    // User identity email + provider  exists, returning Email
                    true => service_clone.get_id(profile, provider),
                }
            })
            .and_then(move |email| service_clone2.create_jwt(email, secret));

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

    fn create_profile(
        &self,
        users_repo: UsersRepoImpl,
        ident_repo: IdentitiesRepoImpl,
        profile_arg: P,
        provider: Provider,
    ) -> Result<i32, ServiceError> {
        let new_user = NewUser::from(profile_arg.clone());
        let profile = profile_arg.clone();

        users_repo
            .create(new_user)
            .map_err(ServiceError::from)
            .map(|user| (profile, user))
            .and_then(move |(profile, user)| {
                ident_repo
                    .create(profile.get_email(), None, provider, user.id)
                    .map_err(ServiceError::from)
                    .map(|u| u.user_id.0)
            })
    }

    fn update_profile(&self, users_repo: UsersRepoImpl, profile: P) -> Result<i32, ServiceError> {
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

    fn create_or_update_profile(&self, profile: P, provider: Provider) -> ServiceFuture<i32> {
        let r2d2_clone = self.r2d2_pool.clone();
        let service = self.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            r2d2_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let users_repo = UsersRepoImpl::new(&conn, Arc::new(SystemACL::new()));
                    let ident_repo = IdentitiesRepoImpl::new(&conn);

                    conn.transaction::<i32, ServiceError, _>(move || {
                        users_repo
                            .email_exists(profile.get_email())
                            .map_err(ServiceError::from)
                            .map(|email_exist| (profile, email_exist))
                            .and_then(move |(profile, email_exist)| -> Result<i32, ServiceError> {
                                match email_exist {
                                    // user doesn't exist, creating user + identity
                                    false => service.create_profile(users_repo, ident_repo, profile, provider),
                                    // user exists, creating identity and filling user info
                                    true => service.update_profile(users_repo, profile),
                                }
                            })
                    })
                })
        }))
    }

    fn email_exists(&self, profile: P, provider: Provider) -> ServiceFuture<bool> {
        let r2d2_clone = self.r2d2_pool.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            r2d2_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let ident_repo = IdentitiesRepoImpl::new(&conn);

                    ident_repo
                        .email_provider_exists(profile.get_email(), provider)
                        .map_err(ServiceError::from)
                })
        }))
    }

    fn get_id(&self, profile: P, provider: Provider) -> ServiceFuture<i32> {
        let r2d2_clone = self.r2d2_pool.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            r2d2_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let ident_repo = IdentitiesRepoImpl::new(&conn);

                    ident_repo
                        .find_by_email_provider(profile.get_email(), provider)
                        .map_err(ServiceError::from)
                        .map(|ident| ident.user_id.0)
                })
        }))
    }
}

impl JWTService for JWTServiceImpl {
    /// Creates new JWT token by email
    fn create_token_email(&self, payload: NewIdentity) -> ServiceFuture<JWT> {
        let r2d2_clone = self.r2d2_pool.clone();
        let jwt_secret_key = self.jwt_config.secret_key.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            r2d2_clone
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let ident_repo = IdentitiesRepoImpl::new(&conn);

                    conn.transaction::<JWT, ServiceError, _>(move || {
                        ident_repo
                            .email_provider_exists(payload.email.to_string(), Provider::Email)
                            .map_err(ServiceError::from)
                            .map(|exists| (exists, payload))
                            .and_then(move |(exists, new_ident)| -> Result<i32, ServiceError> {
                                match exists {
                                    // email does not exist
                                    false => Err(ServiceError::NotFound),
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
                                                    false => Err(ServiceError::IncorrectCredentials),
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
                                    .and_then(|t| Ok(JWT { token: t }))
                            })
                    })
                })
        }))
    }

    /// https://developers.google.com/identity/protocols/OpenIDConnect#validatinganidtoken
    /// Creates new JWT token by google
    fn create_token_google(&self, oauth: ProviderOauth) -> ServiceFuture<JWT> {
        let url = self.google_config.info_url.clone();
        let mut headers = Headers::new();
        headers.set(Authorization(Bearer { token: oauth.token }));
        let jwt_secret_key = self.jwt_config.secret_key.clone();
        <JWTServiceImpl as ProfileService<GoogleProfile>>::create_token(self, Provider::Google, jwt_secret_key, url, Some(headers))
    }

    /// https://developers.facebook.com/docs/facebook-login/manually-build-a-login-flow
    /// Creates new JWT token by facebook
    fn create_token_facebook(&self, oauth: ProviderOauth) -> ServiceFuture<JWT> {
        let info_url = self.facebook_config.info_url.clone();
        let url = format!(
            "{}?fields=first_name,last_name,gender,email,name&access_token={}",
            info_url, oauth.token
        );
        let jwt_secret_key = self.jwt_config.secret_key.clone();
        <JWTServiceImpl as ProfileService<FacebookProfile>>::create_token(self, Provider::Facebook, jwt_secret_key, url, None)
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
