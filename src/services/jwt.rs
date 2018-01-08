use std::sync::Arc;

use futures::future;
use futures::Future;

use error::Error as ApiError;
use models::jwt::JWT;
use payloads::user::{NewUser, UpdateUser};
use payloads::jwt::ProviderOauth;
use repos::users::UsersRepo;
use jsonwebtoken::{encode, Header, Algorithm};

/// JWT services, responsible for JsonWebToken operations
pub struct JWTService {
    pub users_repo: Arc<UsersRepo>,
    pub secret_key: String,
}

impl JWTService {
    /// Creates new JWT token by email
    pub fn create_token_email(&self, user: NewUser) -> Box<Future<Item = JWT, Error = ApiError>> {
        let insert_repo = self.users_repo.clone();
        let jwt_repo = self.jwt_repo.clone();

        let future = self.users_repo.email_exists(user.email.to_string())
            .map(|exists| (user, exists))
            .and_then(|(user, exists)| match exists {
                true => future::ok(user),
                false => insert_repo.create(user).map(|_| user),
            })
            .and_then(|user| 
                let token = encode(&Header::default(), &user, self.secret_key.as_ref())
                    .map_err(|err| {Err(err)})?;
                future::ok(JWT { token: token})
            );

        Box::new(future)
    }

    /// Creates new JWT token by google
    pub fn create_token_google(&self, oauth: ProviderOauth) -> Box<Future<Item = JWT, Error = ApiError>> {
        let jwt_repo = self.jwt_repo.clone();
        let token = encode(&Header::default(), &oauth, self.secret_key.as_ref())
            .map_err(|err| {Err(err)})?;
        let future = future::ok(JWT { token: token});
        Box::new(future)
    }

    /// Creates new JWT token by facebook
    pub fn create_token_facebook(&self, oauth: ProviderOauth) -> Box<Future<Item = JWT, Error = ApiError>> {
        let jwt_repo = self.jwt_repo.clone();
        let token = encode(&Header::default(), &oauth, self.secret_key.as_ref())
            .map_err(|err| {Err(err)})?;
        let future = future::ok(JWT { token: token});
        Box::new(future)
    }

}
