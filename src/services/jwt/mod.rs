pub mod model;

use std::sync::Arc;
use std::str;

use futures::future;
use futures::{Future, IntoFuture};
use futures_cpupool::CpuPool;
use hyper::{Method, Headers};
use hyper::header::{Authorization, Bearer};
use jsonwebtoken::{encode, Header};
use sha3::{Digest, Sha3_256};
use base64::decode;
use serde;


use models::jwt::{JWT, ProviderOauth};
use models::user::{NewUser};
use models::identity::{Provider, NewIdentity};
use repos::identities::{IdentitiesRepo, IdentitiesRepoImpl};
use repos::users::{UsersRepo, UsersRepoImpl};
use http::client::ClientHandle;
use config::JWT as JWTConfig;
use config::OAuth;
use config::Config;
use super::types::ServiceFuture;
use super::error::Error;
use repos::types::DbPool;
use self::model::{GoogleProfile, FacebookProfile, JWTPayload, Email, IntoUser};
use repos::acl::{SystemACL};





/// JWT services, responsible for JsonWebToken operations
pub trait JWTService {
    /// Creates new JWT token by email
    fn create_token_email(&self, payload: NewIdentity) -> ServiceFuture<JWT>;
    /// Creates new JWT token by google
    fn create_token_google(&self, oauth: ProviderOauth) -> ServiceFuture<JWT>;
    /// Creates new JWT token by facebook
    fn create_token_facebook(&self, oauth: ProviderOauth) -> ServiceFuture<JWT>;
    /// Verifies password
    fn password_verify(db_hash: String, clear_password: String) -> Result<bool, Error>;
}

/// JWT services, responsible for JsonWebToken operations
#[derive(Clone)]
pub struct JWTServiceImpl <U:'static + UsersRepo + Clone, I: 'static + IdentitiesRepo+ Clone> {
    pub users_repo: U,
    pub ident_repo: I,
    pub http_client: ClientHandle,
    pub google_config: OAuth,
    pub facebook_config: OAuth,
    pub jwt_config: JWTConfig,
}

impl JWTServiceImpl<UsersRepoImpl, IdentitiesRepoImpl> {
    pub fn new(db_pool: DbPool, cpu_pool:CpuPool, http_client: ClientHandle, config: Config) -> Self {
        let users_repo = UsersRepoImpl::new(db_pool.clone(), cpu_pool.clone(), Arc::new(SystemACL::new()));
        let ident_repo = IdentitiesRepoImpl::new(db_pool, cpu_pool);
        Self {
            users_repo: users_repo,
            ident_repo: ident_repo,
            http_client: http_client,
            google_config: config.google,
            facebook_config: config.facebook,
            jwt_config: config.jwt,
        }
    }
}

trait ProfileService<P: Email> {
    fn create_token(&self, provider: Provider, secret: String, info_url: String, headers: Option<Headers>)  -> ServiceFuture<JWT> ;

    fn get_profile(&self, url: String, headers: Option<Headers>) -> ServiceFuture<P>;

    fn email_exists(&self, profile: P, provider: Provider) -> ServiceFuture<bool> ;

    fn create_jwt(&self, id: i32, secret: String) -> ServiceFuture<JWT> {
        let tokenpayload = JWTPayload::new(id);
        Box::new(
            encode(&Header::default(), &tokenpayload, secret.as_ref())
                .map_err(|_| Error::Parse(format!("Couldn't encode jwt: {:?}.", tokenpayload)))
                .into_future()
                .and_then(|t| future::ok( JWT { token: t }))
        )
    }

    fn create_profile(&self, profile: P, provider: Provider) -> ServiceFuture<i32>;

    fn update_profile(&self, profile: P) -> ServiceFuture<i32>;

    fn create_or_update_profile(&self, profile: P, provider: Provider) -> ServiceFuture<i32>;

    fn get_id(&self, profile: P, provider: Provider) -> ServiceFuture<i32>;
}


impl<P, U, I> ProfileService<P> for JWTServiceImpl<U, I> 
    where   P: Email + Clone + 'static,
            U: UsersRepo + Clone,
            I: IdentitiesRepo + Clone,
            NewUser: From<P>,
            P: for<'a> serde::Deserialize<'a>,
            P: IntoUser {

    fn create_token(&self, provider: Provider, secret: String, info_url: String, headers: Option<Headers>)  -> ServiceFuture<JWT> {
        let service = self.clone();
        let service_clone = self.clone();
        let service_clone2 = self.clone();
        let provider_clone = provider.clone();

        let future = 
            service
                .get_profile(info_url, headers)
                .and_then(move |profile| {
                    let profile_clone = profile.clone();
                    service.email_exists(profile, provider_clone)
                        .map(|exists| (exists, profile_clone))
                })
                .and_then(
                    move |(exists, profile)| -> ServiceFuture<i32>{
                        match exists {
                            // identity email + provider  doesn't exist
                            false => service_clone.create_or_update_profile(profile, provider),
                            // User identity email + provider  exists, returning Email
                            true => service_clone.get_id(profile, provider),
                        }
                    }
                )
                .and_then(move |email| {
                    service_clone2.create_jwt(email, secret)
                });

        Box::new(future)
    }
    
    fn get_profile(&self, url: String, headers: Option<Headers>) -> ServiceFuture<P> {
        Box::new(self.http_client.request::<P>(Method::Get, url, None, headers)
                    .map_err(|e| Error::HttpClient(format!("Failed to receive user info from provider. {}", e.to_string()))))
    }
    
    
    fn create_profile(&self, profile: P, provider: Provider) -> ServiceFuture<i32> {
        let new_user = NewUser::from(profile.clone());
        let users_repo = self.users_repo.clone();
        let ident_repo = self.ident_repo.clone();
        Box::new(
            users_repo
                .create(new_user)
                .map_err(Error::from)
                .map(|user| (profile, user))
                .and_then(move |(profile, user)| {
                    ident_repo
                        .create(profile.get_email(), None, provider, user.id)
                        .map_err(Error::from)
                        .map(|u| u.user_id)
                })
        )
    }

    fn update_profile(&self, profile: P) -> ServiceFuture<i32> {
        let users_repo = self.users_repo.clone();
        Box::new(
            users_repo
                .find_by_email(profile.get_email())
                .map_err(Error::from)
                .map(|user| (profile, user))
                .and_then(move |(profile, user)| {
                    let update_user = profile.merge_into_user(user.clone());
                    Box::new(
                        users_repo.update(user.id, update_user)
                        .map_err(Error::from)
                        .map(|u| u.id)
                    )
                }                                                
            ))
    }

    fn create_or_update_profile(&self, profile: P, provider: Provider) -> ServiceFuture<i32> {
        let users_repo = self.users_repo.clone();
        let service = self.clone();
        Box::new(users_repo
            .email_exists(profile.get_email())
            .map_err(Error::from)
            .map(|email_exist| (profile, email_exist))
            .and_then(move |(profile, email_exist)| ->  ServiceFuture<i32> {
                match email_exist {
                    // user doesn't exist, creating user + identity
                    false => service.create_profile(profile, provider),
                    // user exists, creating identity and filling user info
                    true => service.update_profile(profile)
            }})
        )
    }

    fn email_exists(&self, profile: P, provider: Provider) -> ServiceFuture<bool> {
        let ident_repo = self.ident_repo.clone();
        Box::new(ident_repo
            .email_provider_exists(profile.get_email(), provider)
            .map_err(Error::from))
    }
    
    fn get_id(&self, profile: P, provider: Provider) -> ServiceFuture<i32> {
        let ident_repo = self.ident_repo.clone();
        Box::new(
            ident_repo
                .find_by_email_provider(profile.get_email(), provider)
                .map_err(Error::from)
                .map(|ident| ident.user_id)
        )
    }
}
 

impl<U: UsersRepo + Clone, I: IdentitiesRepo + Clone> JWTService for JWTServiceImpl<U, I> {
    /// Creates new JWT token by email
     fn create_token_email(
        &self,
        payload: NewIdentity,
    ) -> ServiceFuture<JWT> {
        let ident_repo = self.ident_repo.clone();
        let jwt_secret_key = self.jwt_config.secret_key.clone();

        Box::new(
            ident_repo
                .email_provider_exists(payload.email.to_string(), Provider::Email)
                .map_err(Error::from)
                .map(|exists| (exists, payload))
                .and_then(
                    move |(exists, new_ident)| -> ServiceFuture<i32> {
                        match exists {
                            // email does not exist
                            false => Box::new(future::err(Error::Validate(validation_errors!({"email": ["email" => "Email or password are incorrect"]})))),
                            // email exists, checking password
                            true => {
                                let new_ident_clone = new_ident.clone();
                                Box::new(
                                    ident_repo
                                        .find_by_email_provider(new_ident.email.clone(), Provider::Email)
                                        .map_err(Error::from)
                                        .and_then (move |identity| 
                                            Self::password_verify(identity.user_password.unwrap().clone(), new_ident.password.clone())
                                        )
                                        .map(move |verified| (verified, new_ident_clone))
                                        .and_then( move |(verified, new_ident)| -> ServiceFuture<i32> {
                                                match verified {
                                                    //password not verified
                                                    false => Box::new(future::err(Error::Validate(validation_errors!({"email": ["email" => "Email or password are incorrect"]})))),
                                                    //password verified
                                                    true => Box::new(ident_repo
                                                                        .find_by_email_provider(new_ident.email, Provider::Email)
                                                                        .map_err(Error::from)
                                                                        .map(|ident| ident.user_id))
                                                }
                                        })
                                )
                            },
                        }
                    }
                )
                .and_then(move |id| {
                    let tokenpayload = JWTPayload::new(id);
                    encode(&Header::default(), &tokenpayload, jwt_secret_key.as_ref())
                        .map_err(|_| Error::Parse(format!("Couldn't encode jwt: {:?}", tokenpayload)))
                        .into_future()
                        .and_then(|t| future::ok(JWT { token: t }))
                })
        )
    }

    /// https://developers.google.com/identity/protocols/OpenIDConnect#validatinganidtoken
    /// Creates new JWT token by google
     fn create_token_google(
        &self,
        oauth: ProviderOauth,
    ) -> ServiceFuture<JWT> {
        let url = self.google_config.info_url.clone();
        let mut headers = Headers::new();
        headers.set( Authorization ( Bearer {
            token: oauth.token
        }));
        let jwt_secret_key = self.jwt_config.secret_key.clone();
        <JWTServiceImpl<U,I> as ProfileService<GoogleProfile>>::create_token(self, Provider::Google, jwt_secret_key, url, Some(headers))
    }

    /// https://developers.facebook.com/docs/facebook-login/manually-build-a-login-flow
    /// Creates new JWT token by facebook
     fn create_token_facebook(
        &self,
        oauth: ProviderOauth,
    ) -> ServiceFuture<JWT> {
        let info_url = self.facebook_config.info_url.clone();
        let url = format!("{}?fields=first_name,last_name,gender,email,name&access_token={}", info_url, oauth.token);
        let jwt_secret_key = self.jwt_config.secret_key.clone();
        <JWTServiceImpl<U,I> as ProfileService<FacebookProfile>>::create_token(self, Provider::Facebook, jwt_secret_key, url, None)
    }

    fn password_verify(db_hash: String, clear_password: String) -> Result<bool, Error> {
        let v: Vec<&str> = db_hash.split('.').collect();
        if v.len() != 2 {
            Err(Error::Validate(validation_errors!({"password": ["password" => "Password in db has wrong format"]})))
        } else {
            let salt = v[1];
            let pass = clear_password + salt;
            let mut hasher = Sha3_256::default();
            hasher.input(pass.as_bytes());
            let out = hasher.result();
            let computed_hash = decode(v[0])
            .map_err(|_| Error::Validate(validation_errors!({"password": ["password" => "Password in db has wrong format"]})))?;
            Ok(computed_hash == &out[..])
        }
    }
}





