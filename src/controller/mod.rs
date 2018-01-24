//! `Controller` is a top layer that handles all http-related
//! stuff like reading bodies, parsing params, forming a response.
//! Basically it provides inputs to `Service` layer and converts outputs
//! of `Service` layer to http responses

pub mod error;
pub mod routes;
pub mod types;
pub mod utils;

use std::sync::Arc;

use futures::Future;
use futures::future;
use hyper::{Get, Post, Put, Delete};
use hyper::server::Request;
use hyper::header::Authorization;
use serde_json;

use self::error::Error;
use services::system::{SystemServiceImpl, SystemService};
use services::users::{UsersServiceImpl, UsersService};
use services::jwt::JWTService;
use repos::users::{UsersRepo};

use models;
use self::utils::parse_body;
use self::types::ControllerFuture;
use self::routes::{Route, RouteParser};
use services::context::Context;

/// Controller handles route parsing and calling `Service` layer
pub struct Controller<U: UsersRepo + 'static> {
    pub route_parser: Arc<RouteParser>,
    pub jwt_service: Arc<JWTService>,
    pub users_repo: Arc<U>
}

macro_rules! serialize_future {
    ($e:expr) => (Box::new($e.map_err(|e| Error::from(e)).and_then(|resp| serde_json::to_string(&resp).map_err(|e| Error::from(e)))))
}

impl<U: UsersRepo + 'static> Controller<U> {
    /// Create a new controller based on services
    pub fn new(
        users_repo: Arc<U>,
        jwt_service: Arc<JWTService>
    ) -> Self {
        let route_parser = Arc::new(routes::create_route_parser());
        Self {
            route_parser,
            jwt_service,
            users_repo,
        }
    }

    /// Handle a request and get future response
    pub fn call(&self, req: Request) -> ControllerFuture
    {
        let headers = req.headers().clone();
        let auth_header = headers.get::<Authorization<String>>();
        let token_payload = auth_header.map (move |auth| {
                auth.0.clone()
            });
        let context = Context {user_email : token_payload};

        match (req.method(), self.route_parser.test(req.path())) {
            // GET /healthcheck
            (&Get, Some(Route::Healthcheck)) =>
                {
                    let system_service = SystemServiceImpl::new();
                    serialize_future!(system_service.healthcheck().map_err(|e| Error::from(e)))
                },

            // GET /users/<user_id>
            (&Get, Some(Route::User(user_id))) => {
                let users_service = UsersServiceImpl::new(self.users_repo.clone(), context);
                serialize_future!(users_service.get(user_id))
            },

            // GET /users/current
            (&Get, Some(Route::Current)) => {
                let users_service = UsersServiceImpl::new(self.users_repo.clone(), context);
                serialize_future!(users_service.current())
            },

            // GET /users
            (&Get, Some(Route::Users)) => {
                if let (Some(from), Some(to)) = parse_query!(req.query().unwrap_or_default(), "from" => i32, "to" => i64) {
                    let users_service = UsersServiceImpl::new(self.users_repo.clone(), context);
                    serialize_future!(users_service.list(from, to))
                } else {
                    Box::new(future::err(Error::UnprocessableEntity("Error parsing request body".to_string())))
                }
            },

            // POST /users
            (&Post, Some(Route::Users)) => {
                let users_service = UsersServiceImpl::new(self.users_repo.clone(), context);
                serialize_future!(
                    parse_body::<models::user::NewUser>(req.body())
                        .map_err(|_| Error::UnprocessableEntity("Error parsing request body".to_string()))
                        .and_then(move |new_user| users_service.create(new_user).map_err(|e| Error::from(e)))
                )
            },

            // PUT /users/<user_id>
            (&Put, Some(Route::User(user_id))) => {
                let users_service = UsersServiceImpl::new(self.users_repo.clone(), context);
                serialize_future!(
                    parse_body::<models::user::UpdateUser>(req.body())
                        .map_err(|_| Error::UnprocessableEntity("Error parsing request body".to_string()))
                        .and_then(move |update_user| users_service.update(user_id, update_user).map_err(|e| Error::from(e)))
                )
            }

            // DELETE /users/<user_id>
            (&Delete, Some(Route::User(user_id))) =>
                {
                    let users_service = UsersServiceImpl::new(self.users_repo.clone(), context);
                    serialize_future!(users_service.deactivate(user_id))
                },

            // POST /jwt/email
            (&Post, Some(Route::JWTEmail)) => {
                let jwt_service = self.jwt_service.clone();
                serialize_future!(
                    parse_body::<models::user::NewUser>(req.body())
                        .map_err(|_| Error::UnprocessableEntity("Error parsing request body".to_string()))
                        .and_then(move |new_user| jwt_service.create_token_email(new_user).map_err(|e| Error::from(e)))
                )
            },

            // POST /jwt/google
            (&Post, Some(Route::JWTGoogle)) =>  {
                let jwt_service = self.jwt_service.clone();
                serialize_future!(
                    parse_body::<models::jwt::ProviderOauth>(req.body())
                        .map_err(|_| Error::UnprocessableEntity("Error parsing request body".to_string()))
                        .and_then(move |oauth| jwt_service.create_token_google(oauth).map_err(|e| Error::from(e)))
                )
            },
            // POST /jwt/facebook
            (&Post, Some(Route::JWTFacebook)) => {
                let jwt_service = self.jwt_service.clone();
                serialize_future!(
                    parse_body::<models::jwt::ProviderOauth>(req.body())
                        .map_err(|_| Error::UnprocessableEntity("Error parsing request body".to_string()))
                        .and_then(move |oauth| jwt_service.create_token_facebook(oauth).map_err(|e| Error::from(e)))
                )
            },


            // Fallback
            _ => Box::new(future::err(Error::NotFound))
        }
    }
}
