//! `Controller` is a top layer that handles all http-related
//! stuff like reading bodies, parsing params, forming a response.
//! Basically it provides inputs to `Service` layer and converts outputs
//! of `Service` layer to http responses

pub mod error;
pub mod routes;
pub mod types;
pub mod utils;

use std::sync::Arc;
use std::str::FromStr;

use futures::Future;
use futures::future;
use hyper::{Get, Post, Put, Delete};
use hyper::server::Request;
use hyper::header::Authorization;
use serde_json;
use futures_cpupool::CpuPool;

use self::error::Error;
use services::system::{SystemServiceImpl, SystemService};
use services::users::{UsersServiceImpl, UsersService};
use services::jwt::{JWTService, JWTServiceImpl};
use repos::types::DbPool;
use repos::acl::{RolesCacheImpl};

use models;
use self::utils::parse_body;
use self::types::ControllerFuture;
use self::routes::{Route, RouteParser};
use http::client::ClientHandle;
use config::Config;

/// Controller handles route parsing and calling `Service` layer
pub struct Controller {
    pub db_pool: DbPool, 
    pub cpu_pool: CpuPool,
    pub route_parser: Arc<RouteParser>,
    pub config : Config,
    pub client_handle: ClientHandle,
    pub roles_cache: RolesCacheImpl
}

macro_rules! serialize_future {
    ($e:expr) => (Box::new($e.map_err(|e| Error::from(e)).and_then(|resp| serde_json::to_string(&resp).map_err(|e| Error::from(e)))))
}

impl Controller {
    /// Create a new controller based on services
    pub fn new(
        db_pool: DbPool, 
        cpu_pool: CpuPool,
        client_handle: ClientHandle,
        config: Config,
        roles_cache: RolesCacheImpl
    ) -> Self {
        let route_parser = Arc::new(routes::create_route_parser());
        Self {
            route_parser,
            db_pool,
            cpu_pool,
            client_handle,
            config,
            roles_cache
        }
    }

    /// Handle a request and get future response
    pub fn call(&self, req: Request) -> ControllerFuture
    {
        let headers = req.headers().clone();
        let auth_header = headers.get::<Authorization<String>>();
        let user_id = auth_header.map (move |auth| {
            auth.0.clone() 
        }).and_then(|id| {
            i32::from_str(&id).ok()
        });

        let cached_roles = self.roles_cache.clone();
        let system_service = SystemServiceImpl::new();
        let users_service = UsersServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), cached_roles, user_id);
        let jwt_service = JWTServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), self.client_handle.clone(), self.config.clone());

        match (req.method(), self.route_parser.test(req.path())) {
            // GET /healthcheck
            (&Get, Some(Route::Healthcheck)) =>
                {
                    serialize_future!(system_service.healthcheck().map_err(|e| Error::from(e)))
                },

            // GET /users/<user_id>
            (&Get, Some(Route::User(user_id))) => {
                serialize_future!(users_service.get(user_id))
            },

            // GET /users/current
            (&Get, Some(Route::Current)) => {
                serialize_future!(users_service.current())
            },

            // GET /users
            (&Get, Some(Route::Users)) => {
                if let (Some(from), Some(to)) = parse_query!(req.query().unwrap_or_default(), "from" => i32, "to" => i64) {
                    serialize_future!(users_service.list(from, to))
                } else {
                    Box::new(future::err(Error::UnprocessableEntity("Error parsing request from gateway body".to_string())))
                }
            },

            // POST /users
            (&Post, Some(Route::Users)) => {
                serialize_future!(
                    parse_body::<models::identity::NewIdentity>(req.body())
                        .map_err(|_| Error::UnprocessableEntity("Error parsing request from gateway body".to_string()))
                        .and_then(move |new_ident| users_service.create(new_ident).map_err(|e| Error::from(e)))
                )
            },

            // PUT /users/<user_id>
            (&Put, Some(Route::User(user_id))) => {
                serialize_future!(
                    parse_body::<models::user::UpdateUser>(req.body())
                        .map_err(|_| Error::UnprocessableEntity("Error parsing request from gateway body".to_string()))
                        .and_then(move |update_user| users_service.update(user_id, update_user).map_err(|e| Error::from(e)))
                )
            }

            // DELETE /users/<user_id>
            (&Delete, Some(Route::User(user_id))) => {
                serialize_future!(users_service.deactivate(user_id))
            },

            // POST /jwt/email
            (&Post, Some(Route::JWTEmail)) => {
                serialize_future!(
                    parse_body::<models::identity::NewIdentity>(req.body())
                        .map_err(|_| Error::UnprocessableEntity("Error parsing request from gateway body".to_string()))
                        .and_then(move |new_ident| jwt_service.create_token_email(new_ident).map_err(|e| Error::from(e)))
                )
            },

            // POST /jwt/google
            (&Post, Some(Route::JWTGoogle)) =>  {
                serialize_future!(
                    parse_body::<models::jwt::ProviderOauth>(req.body())
                        .map_err(|_| Error::UnprocessableEntity("Error parsing request from gateway body".to_string()))
                        .and_then(move |oauth| jwt_service.create_token_google(oauth).map_err(|e| Error::from(e)))
                )
            },
            // POST /jwt/facebook
            (&Post, Some(Route::JWTFacebook)) => {
                serialize_future!(
                    parse_body::<models::jwt::ProviderOauth>(req.body())
                        .map_err(|_| Error::UnprocessableEntity("Error parsing request from gateway body".to_string()))
                        .and_then(move |oauth| jwt_service.create_token_facebook(oauth).map_err(|e| Error::from(e)))
                )
            },


            // Fallback
            _ => Box::new(future::err(Error::NotFound))
        }
    }
}
