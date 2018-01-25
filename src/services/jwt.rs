use std::sync::Arc;

use futures::future;
use futures::{Future, IntoFuture};
use hyper::mime::{APPLICATION_WWW_FORM_URLENCODED};
use hyper::{Method, Headers};
use hyper::header::{Authorization, Bearer, ContentLength, ContentType};

use models::jwt::{JWT, ProviderOauth};
use models::user::NewUser;
use repos::users::UsersRepo;
use http::client::ClientHandle;
use jsonwebtoken::{encode, Header};
use config::JWT as JWTConfig;
use config::OAuth;
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
  verified_email: bool
}

#[derive(Serialize, Deserialize)]
struct GoogleToken
{
  access_token: String,
  token_type: String,
  expires_in: i32
}

#[derive(Serialize, Deserialize)]
struct FacebookID {
    id: String,
    email: String,
    gender: String,
    first_name: String,
    last_name: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct FacebookToken
{
  access_token: String,
  token_type: String,
  expires_in: i32
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

pub trait JWTService {

    /// Creates new JWT token by email
    fn create_token_email(&self, payload: NewUser) -> ServiceFuture<JWT>;

    /// Creates new JWT token by google
    fn create_token_google(&self, oauth: ProviderOauth) -> ServiceFuture<JWT>;

    /// Creates new JWT token by facebook
    fn create_token_facebook(&self, oauth: ProviderOauth) -> ServiceFuture<JWT>;

}
/// JWT services, responsible for JsonWebToken operations
pub struct JWTServiceImpl <U:'static + UsersRepo> {
    pub users_repo: Arc<U>,
    pub http_client: ClientHandle,
    pub google_settings: OAuth,
    pub facebook_settings: OAuth,
    pub jwt_settings: JWTConfig,
}

impl<U: UsersRepo> JWTService for JWTServiceImpl<U> {
    /// Creates new JWT token by email
     fn create_token_email(
        &self,
        payload: NewUser,
    ) -> ServiceFuture<JWT> {
        let jwt_secret_key = self.jwt_settings.secret_key.clone();

        Box::new(
            self.users_repo
                .verify_password(payload.email.to_string(), payload.password.clone())
                .map_err(Error::from)
                .map(|exists| (exists, payload))
                .and_then(
                    move |(exists, user)| -> ServiceFuture<NewUser> {
                        match exists {
                            false => Box::new(future::err(Error::Validate(validation_errors!({"email": ["email" => "Email or password are incorrect"]})))),
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
     fn create_token_google(
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

        let exchange_code_to_token_url = format!("{}", code_to_token_url );
        let body = format!("code={}&redirect_uri={}&client_id={}&client_secret={}&scope=&grant_type=authorization_code",
            oauth.code,
            redirect_url,
            client_id,
            client_secret
            );
        
        let mut headers =  Headers::new();
        headers.set(ContentLength(body.len() as u64 ) );
        headers.set(ContentType(APPLICATION_WWW_FORM_URLENCODED));

        Box::new(
            http_client.request::<GoogleToken>(Method::Post, exchange_code_to_token_url, Some(body), Some(headers))
                .map_err(|e| Error::HttpClient(format!("Failed to connect to google oauth. {}", e.to_string())))
                .and_then(move |token| {
                    let mut headers = Headers::new();
                    headers.set( Authorization ( Bearer {
                                token: token.access_token
                            }));
                    http_client.request::<GoogleID>(Method::Get, info_url, None, Some(headers))
                        .map_err(|e| Error::HttpClient(format!("Failed to receive user info from google. {}", e.to_string())))
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
     fn create_token_facebook(
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
                .map_err(|e| Error::HttpClient(format!("Failed to connect to facebook oauth. {}", e.to_string())))
                .and_then(move |token| {
                    let url = format!("{}?fields=first_name,last_name,gender,email,name&access_token={}", info_url, token.access_token);
                    http_client.request::<FacebookID>(Method::Get, url, None, None)
                        .map_err(|e| Error::HttpClient(format!("Failed to receive user info from facebook. {}", e.to_string())))
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
