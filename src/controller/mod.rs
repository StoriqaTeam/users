pub mod error;
pub mod routes;
pub mod types;

use std::sync::Arc;

use futures::Future;
use futures::future;
use hyper;
use hyper::{Get, Post, Put, Delete};
use hyper::server::{Service, Request};

use error::Error;
use services::system::SystemService;
use services::users::UsersService;
use services::jwt::JWTService;
use utils::http::{response_with_error, response_with_json, parse_body};
use serde_json;

use payloads;
use self::types::ControllerFuture;
use self::routes::Route;

/// Controller contains all services and `Router`
pub struct Controller {
    pub route_parser: Arc<RouteParser>,
    pub system_service: Arc<SystemService>,
    pub users_service: Arc<UsersService>,
    pub jwt_service: Arc<JWTService>
}

macro_rules! serialize_future {
    ($e:expr) => (Box::new($e.map_err(|e| Error::from(e)).and_then(|resp| serde_json::to_string(&resp).map_err(|e| Error::from(e)))))
}

impl Controller {
    fn new(
        system_service: Arc<SystemService>,
        users_service: Arc<UsersService>,
        jwt_service: Arc<JWTService>
    ) -> Self {
        let route_parser = Arc::new(routes::create_route_parser());
        Controller {
            route_parser,
            system_service,
            users_service,
            jwt_service,
        }
    }

    pub fn call(&self, req: Request) -> ControllerFuture
    {
        match (req.method(), self.route_parser.test(req.path())) {
            // GET /healthcheck
            (&Get, Some(Route::Healthcheck)) =>
                serialize_future!(self.system_service.healthcheck().map_err(|e| ApiError::from(e))),

            // GET /users/<user_id>
            (&Get, Some(Route::User(user_id))) =>
                serialize_future!(self.users_service.get(user_id)),

            // GET /users
            (&Get, Some(Route::Users)) => {
                if let (Some(from), Some(to)) = parse_query!(req.query().unwrap_or_default(), "from" => i32, "to" => i64) {
                    serialize_future!(self.users_service.list(from, to))
                } else {
                    Box::new(future::err(ApiError::UnprocessableEntity))
                }
            },

            // POST /users
            (&Post, Some(Route::Users)) => {
                let users_service = self.users_service.clone();
                serialize_future!(
                    parse_body::<payloads::user::NewUser>(req)
                        .map_err(|_| ApiError::UnprocessableEntity)
                        .and_then(move |new_user| users_service.create(new_user).map_err(|e| ApiError::from(e)))
                )
            },

            // PUT /users/<user_id>
            (&Put, Some(Route::User(user_id))) => {
                let users_service = self.users_service.clone();
                serialize_future!(
                    parse_body::<payloads::user::UpdateUser>(req)
                        .map_err(|_| ApiError::UnprocessableEntity)
                        .and_then(move |update_user| users_service.update(user_id, update_user).map_err(|e| ApiError::from(e)))
                )
            }

            // DELETE /users/<user_id>
            (&Delete, Some(Route::User(user_id))) =>
                serialize_future!(self.users_service.deactivate(user_id)),

            // POST /jwt/email
            (&Post, Some(Route::JWTEmail)) => {
                let jwt_service = self.jwt_service.clone();
                serialize_future!(
                    parse_body::<payloads::user::NewUser>(req)
                        .map_err(|_| ApiError::UnprocessableEntity)
                        .and_then(move |new_user| jwt_service.create_token_email(new_user).map_err(|e| ApiError::from(e)))
                )
            },

            // POST /jwt/google
            (&Post, Some(Route::JWTGoogle)) =>  {
                let jwt_service = self.jwt_service.clone();
                serialize_future!(
                    parse_body::<payloads::jwt::ProviderOauth>(req)
                        .map_err(|_| ApiError::UnprocessableEntity)
                        .and_then(move |oauth| jwt_service.create_token_google(oauth).map_err(|e| ApiError::from(e)))
                )
            },
            // POST /jwt/facebook
            (&Post, Some(Route::JWTFacebook)) => {
                let jwt_service = self.jwt_service.clone();
                serialize_future!(
                    parse_body::<payloads::jwt::ProviderOauth>(req)
                        .map_err(|_| ApiError::UnprocessableEntity)
                        .and_then(move |oauth| jwt_service.create_token_facebook(oauth).map_err(|e| ApiError::from(e)))
                )
            },


            // Fallback
            _ => Box::new(future::err(ApiError::NotFound))
        }
    }
}
