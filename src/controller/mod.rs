//! `Controller` is a top layer that handles all http-related
//! stuff like reading bodies, parsing params, forming a response.
//! Basically it provides inputs to `Service` layer and converts outputs
//! of `Service` layer to http responses

extern crate std;

pub mod error;
pub mod routes;
pub mod types;
pub mod utils;

use std::sync::Arc;
use std::str::FromStr;

use futures::Future;
use futures::future;
use futures::IntoFuture;
use hyper::{Delete, Get, Post, Put};
use hyper::server::Request;
use hyper::header::Authorization;
use serde_json;
use serde::ser::Serialize;
use futures_cpupool::CpuPool;

use self::error::ControllerError;
use services::system::{SystemService, SystemServiceImpl};
use services::users::{UsersService, UsersServiceImpl};
use services::jwt::{JWTService, JWTServiceImpl};
use services::user_roles::{UserRolesService, UserRolesServiceImpl};
use repos::types::DbPool;
use repos::acl::RolesCacheImpl;

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
    pub config: Config,
    pub client_handle: ClientHandle,
    pub roles_cache: RolesCacheImpl,
}

fn serialize_future<T, E, F>(f: F) -> ControllerFuture
where
    F: IntoFuture<Item = T, Error = E> + 'static,
    E: 'static,
    ControllerError: std::convert::From<E>,
    T: Serialize,
{
    Box::new(
        f.into_future()
            .map_err(ControllerError::from)
            .and_then(|resp| serde_json::to_string(&resp).map_err(|e| e.into())),
    )
}

impl Controller {
    /// Create a new controller based on services
    pub fn new(db_pool: DbPool, cpu_pool: CpuPool, client_handle: ClientHandle, config: Config, roles_cache: RolesCacheImpl) -> Self {
        let route_parser = Arc::new(routes::create_route_parser());
        Self {
            route_parser,
            db_pool,
            cpu_pool,
            client_handle,
            config,
            roles_cache,
        }
    }

    /// Handle a request and get future response
    pub fn call(&self, req: Request) -> ControllerFuture {
        let headers = req.headers().clone();
        let auth_header = headers.get::<Authorization<String>>();
        let user_id = auth_header
            .map(move |auth| auth.0.clone())
            .and_then(|id| i32::from_str(&id).ok());

        let cached_roles = self.roles_cache.clone();
        let system_service = SystemServiceImpl::new();
        let users_service = UsersServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            cached_roles,
            user_id,
        );
        let jwt_service = JWTServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            self.client_handle.clone(),
            self.config.clone(),
        );
        let user_roles_service = UserRolesServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone());

        match (req.method(), self.route_parser.test(req.path())) {
            // GET /healthcheck
            (&Get, Some(Route::Healthcheck)) => serialize_future(
                system_service
                    .healthcheck()
            ),

            // GET /users/<user_id>
            (&Get, Some(Route::User(user_id))) => serialize_future(users_service.get(user_id)),

            // GET /users/current
            (&Get, Some(Route::Current)) => serialize_future(users_service.current()),

            // GET /users
            (&Get, Some(Route::Users)) => {
                if let (Some(from), Some(to)) = parse_query!(req.query().unwrap_or_default(), "from" => i32, "to" => i64) {
                    serialize_future(users_service.list(from, to))
                } else {
                    Box::new(future::err(ControllerError::UnprocessableEntity(
                        format_err!("Error parsing request from gateway body")
                    )))
                }
            }

            // POST /users
            (&Post, Some(Route::Users)) => serialize_future(
                parse_body::<models::identity::NewIdentity>(req.body())
                    .map_err(|e| error::ControllerError::UnprocessableEntity(e.into()))
                    .and_then(move |new_ident| {
                        let checked_new_ident = models::identity::NewIdentity {
                            email: new_ident.email.to_lowercase(),
                            password: new_ident.password,
                        };

                        users_service
                            .create(checked_new_ident)
                            .map_err(ControllerError::from)
                    }),
            ),

            // PUT /users/<user_id>
            (&Put, Some(Route::User(user_id))) => serialize_future(
                parse_body::<models::user::UpdateUser>(req.body())
                    .map_err(|e| error::ControllerError::UnprocessableEntity(e.into()))
                    .and_then(move |update_user| {
                        users_service
                            .update(user_id, update_user)
                            .map_err(ControllerError::from)
                    }),
            ),

            // DELETE /users/<user_id>
            (&Delete, Some(Route::User(user_id))) => serialize_future(users_service.deactivate(user_id)),

            // POST /jwt/email
            (&Post, Some(Route::JWTEmail)) => serialize_future(
                parse_body::<models::identity::NewIdentity>(req.body())
                    .map_err(|e| error::ControllerError::UnprocessableEntity(e.into()))
                    .and_then(move |new_ident| {
                        let checked_new_ident = models::identity::NewIdentity {
                            email: new_ident.email.to_lowercase(),
                            password: new_ident.password,
                        };

                        jwt_service
                            .create_token_email(checked_new_ident)
                            .map_err(ControllerError::from)
                    }),
            ),

            // POST /jwt/google
            (&Post, Some(Route::JWTGoogle)) => serialize_future(
                parse_body::<models::jwt::ProviderOauth>(req.body())
                    .map_err(|e| error::ControllerError::UnprocessableEntity(e.into()))
                    .and_then(move |oauth| {
                        jwt_service
                            .create_token_google(oauth)
                            .map_err(ControllerError::from)
                    }),
            ),
            // POST /jwt/facebook
            (&Post, Some(Route::JWTFacebook)) => serialize_future(
                parse_body::<models::jwt::ProviderOauth>(req.body())
                    .map_err(|e| error::ControllerError::UnprocessableEntity(e.into()))
                    .and_then(move |oauth| {
                        jwt_service
                            .create_token_facebook(oauth)
                            .map_err(ControllerError::from)
                    }),
            ),

            // GET /user_role/<user_role_id>
            (&Get, Some(Route::UserRole(user_role_id))) => serialize_future(user_roles_service.get(user_role_id)),

            // POST /user_roles
            (&Post, Some(Route::UserRoles)) => serialize_future(
                parse_body::<models::NewUserRole>(req.body())
                    .map_err(|_| ControllerError::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |new_store| {
                        user_roles_service
                            .create(new_store)
                            .map_err(ControllerError::from)
                    }),
            ),

            // DELETE /user_roles/<user_role_id>
            (&Delete, Some(Route::UserRoles)) => serialize_future(
                parse_body::<models::OldUserRole>(req.body())
                    .map_err(|_| ControllerError::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .and_then(move |old_role| {
                        user_roles_service
                            .delete(old_role)
                            .map_err(ControllerError::from)
                    }),
            ),

            // Fallback
            _ => Box::new(future::err(ControllerError::NotFound)),
        }
    }
}
