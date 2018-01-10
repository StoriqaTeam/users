use std::sync::Arc;

use futures::future;
use futures::{IntoFuture,Future};

use error::Error as ApiError;
use models::jwt::JWT;
use payloads::user::NewUser;
use payloads::jwt::ProviderOauth;
use repos::users::UsersRepo;
use ::client::ClientHandle;
use hyper::Method;
use jsonwebtoken::{encode, Header};

/// JWT services, responsible for JsonWebToken operations
pub struct JWTService {
    pub users_repo: Arc<UsersRepo>,
    pub secret_key: String,
    pub http_client: ClientHandle
}

impl JWTService {
    /// Creates new JWT token by email
    pub fn create_token_email(&self, payload: NewUser) -> Box<Future<Item = JWT, Error = ApiError>> {
        let insert_repo = self.users_repo.clone();
        let secret_ket = self.secret_key.clone();

        let future = self.users_repo.email_exists(payload.email.to_string())
            .map(|exists| (exists, payload))
            .and_then(move |(exists, user)| -> Box<Future<Item = NewUser, Error = ApiError>> {
                match exists {
                    false => Box::new(insert_repo.create(user.clone()).map(|_| user)),
                    true => Box::new(future::ok(user)),
                }})
            .and_then(move |u| {
                    encode(&Header::default(), &u, secret_ket.as_ref())
                        .map_err(|_e| ApiError::UnprocessableEntity)
                        .into_future()
                        .and_then(|t| future::ok(JWT { token: t}) )
            });
        

        Box::new(future)
    }

    /// Creates new JWT token by google
    pub fn create_token_google(&self, oauth: ProviderOauth) -> Box<Future<Item = JWT, Error = ApiError>> {
        let url = format!("googleapis.com");
        let user = json!({"token": oauth.token});
        let body: String = user.to_string();
        let secret_key = self.secret_key.clone();

        let future =self.http_client.request::<NewUser>(Method::Post, url, Some(body))
            .and_then(move |u| {
                encode(&Header::default(), &u, secret_key.as_ref())
                    .map_err(|_e| ApiError::UnprocessableEntity)
                    .into_future()
                    .and_then(|t| future::ok( JWT { token: t}) )
            });
        
        Box::new(future)
    }

    /// Creates new JWT token by facebook
    pub fn create_token_facebook(&self, oauth: ProviderOauth) -> Box<Future<Item = JWT, Error = ApiError>> {
        let url = format!("facebook.com");
        let user = json!({"token": oauth.token});
        let body: String = user.to_string();
        let secret_key = self.secret_key.clone();

        let future =self.http_client.request::<NewUser>(Method::Post, url, Some(body))
            .and_then(move |u| {
                encode(&Header::default(), &u, secret_key.as_ref())
                    .map_err(|_e| ApiError::UnprocessableEntity)
                    .into_future()
                    .and_then(|t| future::ok( JWT { token: t}) )
            });
        
        Box::new(future)
    }

}
