use std::sync::Arc;

use futures::future;
use futures::{Future, IntoFuture};

use models::jwt::JWT;
use payloads::user::NewUser;
use payloads::jwt::ProviderOauth;
use repos::users::UsersRepo;
use client::ClientHandle;
use hyper::{Method, Headers};
use hyper::header::{Authorization, Bearer};
use jsonwebtoken::{encode, Header};
use settings::JWT as JWTSettings;
use settings::OAuth;
use super::types::ServiceFuture;
use super::error::Error;

#[derive(Serialize, Deserialize)]
struct GoogleID {
  family_name: String,
  name: String,
  picture: String,
  email: String,
  given_name: String,
  id: String,
  hd: String,
  verified_email: String
}

#[derive(Serialize, Deserialize)]
struct GoogleToken
{
  access_token: String,
  refresh_token: String, 
  token_type: String,
  expires_in: String
}

#[derive(Serialize, Deserialize)]
struct FacebookID {
    email: String,
    first_name: String,
    gender: String,
    id: String,
    last_name: String,
    link: String,
    locale: String,
    name: String,
    timezone: String,
    updated_time: String,
    verified: String,
}

#[derive(Serialize, Deserialize)]
struct FacebookToken
{
  access_token: String, 
  token_type: String,
  expires_in: String
}

#[derive(Serialize, Deserialize, Debug)]
struct JWTPayload {
    user_email: String,
}

impl JWTPayload {
    fn new<S: Into<String>>(email: S) -> Self {
        Self {
            user_email: email.into(),
        }
    }
}

/// JWT services, responsible for JsonWebToken operations
pub struct JWTService {
    pub users_repo: Arc<UsersRepo>,
    pub http_client: ClientHandle,
    pub google_settings: OAuth,
    pub facebook_settings: OAuth,
    pub jwt_settings: JWTSettings,
}

impl JWTService {
    /// Creates new JWT token by email
    pub fn create_token_email(
        &self,
        payload: NewUser,
    ) -> ServiceFuture<JWT> {
        let insert_repo = self.users_repo.clone();
        let jwt_secret_key = self.jwt_settings.secret_key.clone();

        Box::new(
            self.users_repo
                .email_exists(payload.email.to_string())
                .map_err(|e| Error::from(e))
                .map(|exists| (exists, payload))
                .and_then(
                    move |(exists, user)| -> ServiceFuture<NewUser> {
                        match exists {
                            false => Box::new(insert_repo.create(user.clone())
                                        .map(|_| user)
                                        .map_err(|e| Error::from(e))),
                            true => Box::new(future::ok(user)),
                        }
                    }
                )
                .and_then(move |u| {
                    let tokenpayload = JWTPayload::new(u.email);
                    encode(&Header::default(), &tokenpayload, jwt_secret_key.as_ref())
                        .map_err(|_| Error::Parse(format!("Couldn't encode jwt: {:?}", tokenpayload)))
                        .into_future()
                        .and_then(|t| future::ok(JWT { token: t }))
                })
        )
    }

    /// https://developers.google.com/identity/protocols/OpenIDConnect#validatinganidtoken
    /// Creates new JWT token by google
    pub fn create_token_google(
        &self,
        oauth: ProviderOauth,
    ) -> ServiceFuture<JWT> {
        
        let jwt_secret_key = self.jwt_settings.secret_key.clone();
        let code_to_token_url = self.google_settings.code_to_token_url.clone();
        let redirect_url = self.google_settings.redirect_url.clone();
        let client_id = self.google_settings.id.clone();
        let client_secret = self.google_settings.key.clone();
        let info_url = self.google_settings.info_url.clone();
        let http_client = self.http_client.clone();
        
        let exchange_code_to_token_url = format!("{}?client_id={}&redirect_uri={}&client_secret={}&code={}&grant_type=authorization_code", 
            code_to_token_url, 
            client_id, 
            redirect_url, 
            client_secret,
            oauth.code);

        Box::new(
            http_client.request::<GoogleToken>(Method::Get, exchange_code_to_token_url, None, None)
                .map_err(|_| Error::HttpClient("Failed to connect to google oauth.".to_string()))
                .and_then(move |token| {
                    let mut headers = Headers::new();
                    headers.set( Authorization ( Bearer {
                                token: token.access_token
                            }));
                    http_client.request::<GoogleID>(Method::Get, info_url, None, Some(headers))
                        .map_err(|_| Error::HttpClient("Failed to receive user info from google.".to_string()))
                })
                .and_then(move |google_id| {
                    let tokenpayload = JWTPayload::new(google_id.email);
                    encode(&Header::default(), &tokenpayload, jwt_secret_key.as_ref())
                        .map_err(|_| Error::Parse(format!("Couldn't encode jwt: {:?}.", tokenpayload)))
                        .into_future()
                        .and_then(|t| future::ok( JWT { token: t }))
                })
        )
    }

    /// https://developers.facebook.com/docs/facebook-login/manually-build-a-login-flow
    /// Creates new JWT token by facebook
    pub fn create_token_facebook(
        &self,
        oauth: ProviderOauth,
    ) -> ServiceFuture<JWT> {
        
        let jwt_secret_key = self.jwt_settings.secret_key.clone();
        let code_to_token_url = self.facebook_settings.code_to_token_url.clone();
        let redirect_url = self.facebook_settings.redirect_url.clone();
        let client_id = self.facebook_settings.id.clone();
        let client_secret = self.facebook_settings.key.clone();
        let info_url = self.facebook_settings.info_url.clone();
        let http_client = self.http_client.clone();
        
        let exchange_code_to_token_url = format!("{}?client_id={}&redirect_uri={}&client_secret={}&code={}", 
            code_to_token_url, 
            client_id, 
            redirect_url, 
            client_secret,
            oauth.code);

        let future = 
            http_client.request::<FacebookToken>(Method::Get, exchange_code_to_token_url, None, None)
                .map_err(|_| Error::HttpClient("Failed to connect to facebook oauth.".to_string()))
                .and_then(move |token| {
                    let url = format!("{}?access_token={}", info_url, token.access_token);
                    http_client.request::<FacebookID>(Method::Get, url, None, None)
                        .map_err(|_| Error::HttpClient("Failed to receive user info from facebook.".to_string()))
                })
                .and_then(move |facebook_id| {
                    let tokenpayload = JWTPayload::new(facebook_id.email);
                    encode(&Header::default(), &tokenpayload, jwt_secret_key.as_ref())
                        .map_err(|_| Error::Parse(format!("Couldn't encode jwt: {:?}.", tokenpayload)))
                        .into_future()
                        .and_then(|t| future::ok( JWT { token: t }))
                });

        Box::new(future)
    }
}