use std::sync::Arc;

use futures::future;
use futures::{IntoFuture,Future};

use error::Error as ApiError;
use models::jwt::JWT;
use payloads::user::NewUser;
use payloads::jwt::ProviderOauth;
use repos::users::UsersRepo;
use jsonwebtoken::{encode, Header};

/// JWT services, responsible for JsonWebToken operations
pub struct JWTService {
    pub users_repo: Arc<UsersRepo>,
    pub secret_key: String,
}

impl JWTService {
    /// Creates new JWT token by email
    pub fn create_token_email(&self, payload: NewUser) -> Box<Future<Item = JWT, Error = ApiError>> {
        let insert_repo = self.users_repo.clone();
        let p = payload.clone();

        let future1 = self.users_repo.email_exists(payload.email.to_string());
        
        let future2 = move |exists| -> Box<Future<Item = (), Error = ApiError>> {
                match exists {
                    false => Box::new(insert_repo.create(p).and_then(|_| future::ok(()))),
                    true => Box::new(future::ok(())),
                }};
        
        let future123 = encode(&Header::default(), &payload, self.secret_key.as_ref())
            .map_err(|_e| ApiError::UnprocessableEntity)
            .into_future()
            .and_then(|t| future::ok(JWT { token: t}) );

        let future = future1
            .and_then(future2)
            .and_then(|_| future123);
        

        Box::new(future)
    }

    /// Creates new JWT token by google
    pub fn create_token_google(&self, oauth: ProviderOauth) -> Box<Future<Item = JWT, Error = ApiError>> {
        let future = encode(&Header::default(), &oauth, self.secret_key.as_ref())
            .map_err(|_e| ApiError::UnprocessableEntity)
            .into_future()
            .and_then(|t| future::ok(JWT { token: t}) );
        Box::new(future)
    }

    /// Creates new JWT token by facebook
    pub fn create_token_facebook(&self, oauth: ProviderOauth) -> Box<Future<Item = JWT, Error = ApiError>> {
        let future = encode(&Header::default(), &oauth, self.secret_key.as_ref())
            .map_err(|_e| ApiError::UnprocessableEntity)
            .into_future()
            .and_then(|t| future::ok(JWT { token: t}) );
        Box::new(future)
    }

}
