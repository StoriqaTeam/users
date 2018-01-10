use std::sync::Arc;

use futures::future;
use futures::{IntoFuture,Future};

use error::Error as ApiError;
use models::jwt::JWT;
use payloads::user::NewUser;
use payloads::jwt::ProviderOauth;
use repos::users::UsersRepo;
use jsonwebtoken::{encode, Header};
use ::client::ClientHandle;
use hyper::Method;

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
        let p = payload.clone();

        let future_email_exist = self.users_repo.email_exists(payload.email.to_string());

        let future_create_new_user = move |exists| -> Box<Future<Item = (), Error = ApiError>> {
                match exists {
                    false => Box::new(insert_repo.create(p).and_then(|_| future::ok(()))),
                    true => Box::new(future::ok(())),
                }};
        
        let future_create_token = encode(&Header::default(), &payload, self.secret_key.as_ref())
            .map_err(|_e| ApiError::UnprocessableEntity)
            .into_future()
            .and_then(|t| future::ok(JWT { token: t}) );

        let future = future_email_exist
            .and_then(future_create_new_user)
            .and_then(|_| future_create_token);
        

        Box::new(future)
    }

    /// Creates new JWT token by google
    pub fn create_token_google(&self, oauth: ProviderOauth) -> Box<Future<Item = JWT, Error = ApiError>> {
        let url = format!("googleapis.com");
        let user = json!({"token": oauth.token});
        let body: String = user.to_string();
        unimplemented!();

//        let future =self.http_client.request::<NewUser>(Method::Post, url, Some(body));

            //.and_then(|u| {
            //    encode(&Header::default(), &u, self.secret_key.as_ref())
            //        .map_err(|_e| ApiError::UnprocessableEntity)
            //        .into_future()
            //        .and_then(|t| future::ok( JWT { token: t}) )
            //})
        
        //Box::new(future)
    }

    /// Creates new JWT token by facebook
    pub fn create_token_facebook(&self, oauth: ProviderOauth) -> Box<Future<Item = JWT, Error = ApiError>> {
        let url = format!("facebook.com");
        let user = json!({"token": oauth.token});
        let body: String = user.to_string();

        unimplemented!();

//        let future =self.http_client.request::<NewUser>(Method::Post, url, Some(body))
//            .and_then(|u| {
//                encode(&Header::default(), &u, self.secret_key.as_ref())
//                    .map_err(|_e| ApiError::UnprocessableEntity)
//                    .into_future()
//                    .and_then(|t| future::ok(JWT { token: t}) )
//            });
//
//        Box::new(future)
    }

}
