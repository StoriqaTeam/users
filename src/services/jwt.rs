use std::sync::Arc;

use futures::future;
use futures::{Future, IntoFuture};

use error::Error as ApiError;
use models::jwt::JWT;
use payloads::user::NewUser;
use payloads::jwt::ProviderOauth;
use repos::users::UsersRepo;
use client::ClientHandle;
use hyper::Method;
use jsonwebtoken::{encode, Header};
use settings::JWT as JWTSettings;
use settings::OAuth;

#[derive(Serialize, Deserialize)]
struct GoogleID {
    iss: String,
    sub: String,
    aud: String,
    iat: String,
    exp: String,

    #[serde(default)] at_hash: Option<String>,
    #[serde(default)] email_verified: Option<String>,
    #[serde(default)] azp: Option<String>,
    #[serde(default)] email: Option<String>,
    #[serde(default)] profile: Option<String>,
    #[serde(default)] picture: Option<String>,
    #[serde(default)] name: Option<String>,
    #[serde(default)] nonce: Option<String>,
    #[serde(default)] hd: Option<String>,
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

#[derive(Serialize, Deserialize)]
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
    ) -> Box<Future<Item = JWT, Error = ApiError>> {
        let insert_repo = self.users_repo.clone();
        let jwt_secret_key = self.jwt_settings.secret_key.clone();

        let future = self.users_repo
            .email_exists(payload.email.to_string())
            .map(|exists| (exists, payload))
            .and_then(
                move |(exists, user)| -> Box<Future<Item = NewUser, Error = ApiError>> {
                    match exists {
                        false => Box::new(insert_repo.create(user.clone()).map(|_| user)),
                        true => Box::new(future::ok(user)),
                    }
                },
            )
            .and_then(move |u| {
                let tokenpayload = JWTPayload::new(u.email);
                encode(&Header::default(), &tokenpayload, jwt_secret_key.as_ref())
                    .map_err(|_e| ApiError::UnprocessableEntity)
                    .into_future()
                    .and_then(|t| future::ok(JWT { token: t }))
            });

        Box::new(future)
    }

    /// https://developers.google.com/identity/protocols/OpenIDConnect#validatinganidtoken
    /// Creates new JWT token by google
    pub fn create_token_google(
        &self,
        oauth: ProviderOauth,
    ) -> Box<Future<Item = JWT, Error = ApiError>> {
        
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

        let future = 
            http_client.request::<GoogleToken>(Method::Get, exchange_code_to_token_url, None)
            .and_then(move |token| {
                let url = format!("{}?id_token={}", info_url, token.access_token);
                http_client.request::<GoogleID>(Method::Get, url, None)
            })
            .and_then(move |google_id| -> Box<Future<Item=JWT, Error= ApiError>>{
                match google_id.email {
                    Some(email) => {
                        let tokenpayload = JWTPayload::new(email);
                        Box::new(
                            encode(&Header::default(), &tokenpayload, jwt_secret_key.as_ref())
                                .map_err(|_e| ApiError::UnprocessableEntity)
                                .into_future()
                                .and_then(|t| future::ok( JWT { token: t })),
                        )
                    }
                    None => Box::new(Err(ApiError::UnprocessableEntity).into_future()),
                }
            });

        Box::new(future)
    }

    /// https://developers.facebook.com/docs/facebook-login/manually-build-a-login-flow
    /// Creates new JWT token by facebook
    pub fn create_token_facebook(
        &self,
        oauth: ProviderOauth,
    ) -> Box<Future<Item = JWT, Error = ApiError>> {
        
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
            http_client.request::<FacebookToken>(Method::Get, exchange_code_to_token_url, None)
            .and_then(move |token| {
                let url = format!("{}?access_token={}", info_url, token.access_token);
                http_client.request::<FacebookID>(Method::Get, url, None)
            })
            .and_then(move |facebook_id| {
                let tokenpayload = JWTPayload::new(facebook_id.email);
                encode(&Header::default(), &tokenpayload, jwt_secret_key.as_ref())
                    .map_err(|_e| ApiError::UnprocessableEntity)
                    .into_future()
                    .and_then(|t| future::ok( JWT { token: t }))
            });

        Box::new(future)
    }
}