use futures::future;
use futures::{Future, IntoFuture};
use futures_cpupool::CpuPool;
use hyper::{Method, Headers};
use hyper::header::{Authorization, Bearer, ContentLength, ContentType};
use hyper::mime::{APPLICATION_WWW_FORM_URLENCODED};
use jsonwebtoken::{encode, Header};


use models::jwt::{JWT, ProviderOauth};
use models::user::NewUser;
use repos::identities::{IdentitiesRepo, IdentitiesRepoImpl};
use repos::users::{UsersRepo, UsersRepoImpl};
use http::client::ClientHandle;
use config::JWT as JWTConfig;
use config::OAuth;
use config::Config;
use super::types::ServiceFuture;
use super::error::Error;
use repos::types::DbPool;

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
pub struct JWTServiceImpl <U:'static + UsersRepo + Clone, I: 'static + IdentitiesRepo+ Clone> {
    pub users_repo: U,
    pub ident_repo: I,
    pub http_client: ClientHandle,
    pub google_config: OAuth,
    pub facebook_config: OAuth,
    pub jwt_config: JWTConfig,
}

impl JWTServiceImpl<UsersRepoImpl, IdentitiesRepoImpl> {
    pub fn new(r2d2_pool: DbPool, cpu_pool:CpuPool, http_client: ClientHandle, config: Config) -> Self {
        let users_repo = UsersRepoImpl::new(r2d2_pool.clone(), cpu_pool.clone());
        let ident_repo = IdentitiesRepoImpl::new(r2d2_pool, cpu_pool);
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
 

impl<U: UsersRepo + Clone, I: IdentitiesRepo + Clone> JWTService for JWTServiceImpl<U, I> {
    /// Creates new JWT token by email
     fn create_token_email(
        &self,
        payload: NewUser,
    ) -> ServiceFuture<JWT> {
        let ident_repo = self.ident_repo.clone();
        let jwt_secret_key = self.jwt_config.secret_key.clone();

        Box::new(
            ident_repo
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
                .and_then(move |pl| {
                    let tokenpayload = JWTPayload::new(pl.email);
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

        let jwt_secret_key = self.jwt_config.secret_key.clone();
        let code_to_token_url = self.google_config.code_to_token_url.clone();
        let redirect_url = self.google_config.redirect_url.clone();
        let client_id = self.google_config.id.clone();
        let client_secret = self.google_config.key.clone();
        let info_url = self.google_config.info_url.clone();
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

        let jwt_secret_key = self.jwt_config.secret_key.clone();
        let code_to_token_url = self.facebook_config.code_to_token_url.clone();
        let redirect_url = self.facebook_config.redirect_url.clone();
        let client_id = self.facebook_config.id.clone();
        let client_secret = self.facebook_config.key.clone();
        let info_url = self.facebook_config.info_url.clone();
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
