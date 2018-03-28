//! `Controller` is a top layer that handles all http-related
//! stuff like reading bodies, parsing params, forming a response.
//! Basically it provides inputs to `Service` layer and converts outputs
//! of `Service` layer to http responses

pub mod routes;
pub mod utils;

use std::str::FromStr;
use std::sync::Arc;

use futures::Future;
use futures::IntoFuture;
use futures::future;
use futures_cpupool::CpuPool;
use hyper::header::Authorization;
use hyper::server::Request;
use hyper::{Delete, Get, Post, Put};
use stq_http::client::ClientHandle;
use stq_http::controller::Controller;
use stq_http::errors::ControllerError;
use stq_http::request_util::ControllerFuture;
use stq_http::request_util::parse_body;
use stq_http::request_util::serialize_future;
use stq_router::RouteParser;
use validator::Validate;

use self::routes::Route;
use config::Config;
use models;
use repos::acl::RolesCacheImpl;
use repos::types::DbPool;
use services::jwt::{JWTService, JWTServiceImpl};
use services::system::{SystemService, SystemServiceImpl};
use services::user_roles::{UserRolesService, UserRolesServiceImpl};
use services::users::{UsersService, UsersServiceImpl};

/// Controller handles route parsing and calling `Service` layer
pub struct ControllerImpl {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub route_parser: Arc<RouteParser<Route>>,
    pub config: Config,
    pub client_handle: ClientHandle,
    pub roles_cache: RolesCacheImpl,
}

impl ControllerImpl {
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
}

impl Controller for ControllerImpl {
    /// Handle a request and get future response
    fn call(&self, req: Request) -> ControllerFuture {
        let headers = req.headers().clone();
        let auth_header = headers.get::<Authorization<String>>();
        let user_id = auth_header.map(move |auth| auth.0.clone()).and_then(|id| i32::from_str(&id).ok());

        let cached_roles = self.roles_cache.clone();
        let system_service = SystemServiceImpl::new();
        let users_service = UsersServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            self.client_handle.clone(),
            cached_roles.clone(),
            user_id,
            self.config.notifications.clone(),
        );
        let jwt_service = JWTServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            self.client_handle.clone(),
            self.config.clone(),
        );
        let user_roles_service = UserRolesServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), cached_roles);

        match (req.method(), self.route_parser.test(req.path())) {
            // GET /healthcheck
            (&Get, Some(Route::Healthcheck)) => {
                debug!("Received healthcheck request");
                serialize_future(system_service.healthcheck())
            }

            // GET /users/<user_id>
            (&Get, Some(Route::User(user_id))) => {
                debug!("Received request to get user info for ID {}", user_id);
                serialize_future(users_service.get(user_id))
            }

            // GET /users/current
            (&Get, Some(Route::Current)) => {
                debug!("Received request to get current user info.");
                serialize_future(users_service.current())
            }

            // GET /users
            (&Get, Some(Route::Users)) => {
                if let (Some(from), Some(count)) = parse_query!(req.query().unwrap_or_default(), "from" => i32, "count" => i64) {
                    debug!("Received request to get {} users starting from {}", count, from);
                    serialize_future(users_service.list(from, count))
                } else {
                    Box::new(future::err(ControllerError::UnprocessableEntity(format_err!(
                        "Error parsing request from gateway body"
                    ))))
                }
            }

            // POST /users
            (&Post, Some(Route::Users)) => serialize_future(
                parse_body::<models::SagaCreateProfile>(req.body())
                    .map_err(|e| ControllerError::UnprocessableEntity(e.into()))
                    .and_then(move |payload| {
                        debug!("Received request to create profile: {:?}", &payload);
                        payload
                            .identity
                            .validate()
                            .map_err(|e| ControllerError::Validate(e))
                            .into_future()
                            .inspect(|_| {
                                debug!("Validation success");
                            })
                            .and_then(move |_| {
                                let checked_new_ident = models::identity::NewIdentity {
                                    email: payload.identity.email.to_lowercase(),
                                    password: payload.identity.password,
                                    provider: payload.identity.provider,
                                    saga_id: payload.identity.saga_id,
                                };

                                users_service.create(checked_new_ident, payload.user).map_err(ControllerError::from)
                            })
                    }),
            ),

            // PUT /users/<user_id>
            (&Put, Some(Route::User(user_id))) => serialize_future(
                parse_body::<models::user::UpdateUser>(req.body())
                    .map_err(|e| ControllerError::UnprocessableEntity(e.into()))
                    .inspect(|payload| {
                        debug!("Received request to update user: {:?}", &payload);
                    })
                    .and_then(move |update_user| {
                        update_user
                            .validate()
                            .map_err(|e| ControllerError::Validate(e))
                            .into_future()
                            .inspect(|_| {
                                debug!("Validation success");
                            })
                            .and_then(move |_| users_service.update(user_id, update_user).map_err(ControllerError::from))
                    }),
            ),

            // DELETE /users/<user_id>
            (&Delete, Some(Route::User(user_id))) => {
                debug!("Received request to deactivate user {}", user_id);
                serialize_future(users_service.deactivate(user_id))
            }

            // DELETE /user_by_saga_id/<user_id>
            (&Delete, Some(Route::UserBySagaId(saga_id))) => {
                debug!("Received request to delete user with saga ID {}", saga_id);
                serialize_future(users_service.delete_by_saga_id(saga_id))
            }

            // POST /jwt/email
            (&Post, Some(Route::JWTEmail)) => serialize_future(
                parse_body::<models::identity::NewEmailIdentity>(req.body())
                    .map_err(|e| ControllerError::UnprocessableEntity(e.into()))
                    .and_then(move |new_ident| {
                        debug!("Received request to authenticate with email: {:?}", &new_ident);
                        new_ident
                            .validate()
                            .map_err(|e| ControllerError::Validate(e))
                            .into_future()
                            .inspect(|_| {
                                debug!("Validation success");
                            })
                            .and_then(move |_| {
                                let checked_new_ident = models::identity::NewEmailIdentity {
                                    email: new_ident.email.to_lowercase(),
                                    password: new_ident.password,
                                };

                                jwt_service.create_token_email(checked_new_ident).map_err(ControllerError::from)
                            })
                    }),
            ),

            // POST /jwt/google
            (&Post, Some(Route::JWTGoogle)) => serialize_future(
                parse_body::<models::jwt::ProviderOauth>(req.body())
                    .map_err(|e| ControllerError::UnprocessableEntity(e.into()))
                    .inspect(|payload| {
                        debug!("Received request to authenticate with Google token: {:?}", &payload);
                    })
                    .and_then(move |oauth| jwt_service.create_token_google(oauth).map_err(ControllerError::from)),
            ),
            // POST /jwt/facebook
            (&Post, Some(Route::JWTFacebook)) => serialize_future(
                parse_body::<models::jwt::ProviderOauth>(req.body())
                    .map_err(|e| ControllerError::UnprocessableEntity(e.into()))
                    .inspect(|payload| {
                        debug!("Received request to authenticate with Facebook token: {:?}", &payload);
                    })
                    .and_then(move |oauth| jwt_service.create_token_facebook(oauth).map_err(ControllerError::from)),
            ),

            // GET /user_roles/<user_id>
            (&Get, Some(Route::UserRole(user_id))) => {
                debug!("Received request to get roles for user {}", user_id);
                serialize_future(user_roles_service.get_roles(user_id))
            }

            // POST /user_roles
            (&Post, Some(Route::UserRoles)) => serialize_future(
                parse_body::<models::NewUserRole>(req.body())
                    .map_err(|_| ControllerError::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .inspect(|payload| {
                        debug!("Received request to create role: {:?}", payload);
                    })
                    .and_then(move |new_role| user_roles_service.create(new_role).map_err(ControllerError::from)),
            ),

            // DELETE /user_roles/<user_role_id>
            (&Delete, Some(Route::UserRoles)) => serialize_future(
                parse_body::<models::OldUserRole>(req.body())
                    .map_err(|_| ControllerError::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .inspect(|payload| {
                        debug!("Received request to remove role: {:?}", payload);
                    })
                    .and_then(move |old_role| user_roles_service.delete(old_role).map_err(ControllerError::from)),
            ),

            // POST /roles/default/<user_id>
            (&Post, Some(Route::DefaultRole(user_id))) => {
                debug!("Received request to add default role for user {}", user_id);
                serialize_future(user_roles_service.create_default(user_id).map_err(ControllerError::from))
            }

            // DELETE /roles/default/<user_id>
            (&Delete, Some(Route::DefaultRole(user_id))) => {
                debug!("Received request to delete default role for user {}", user_id);
                serialize_future(user_roles_service.delete_default(user_id).map_err(ControllerError::from))
            }

            // POST /users/password_reset/request
            (&Post, Some(Route::PasswordResetRequest)) => serialize_future(
                parse_body::<models::ResetRequest>(req.body())
                    .map_err(|_| ControllerError::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .inspect(|payload| {
                        debug!("Received request to start password reset: {:?}", payload);
                    })
                    .and_then(move |reset_req| {
                        reset_req
                            .validate()
                            .map_err(|e| ControllerError::Validate(e))
                            .into_future()
                            .and_then(move |_| users_service.password_reset_request(reset_req.email).map_err(ControllerError::from))
                    }),
            ),

            // POST /users/password_reset/apply
            (&Post, Some(Route::PasswordResetApply)) => serialize_future(
                parse_body::<models::ResetApply>(req.body())
                    .map_err(|_| ControllerError::UnprocessableEntity(format_err!("Error parsing request from gateway body")))
                    .inspect(|payload| {
                        debug!("Received request to complete password reset: {:?}", payload);
                    })
                    .and_then(move |reset_apply| {
                        reset_apply
                            .validate()
                            .map_err(|e| ControllerError::Validate(e))
                            .into_future()
                            .and_then(move |_| {
                                users_service
                                    .password_reset_apply(reset_apply.token, reset_apply.password)
                                    .map_err(ControllerError::from)
                            })
                    }),
            ),

            // POST /email_verify/resend/<email>
            (&Post, Some(Route::EmailVerifyResend(email))) => serialize_future(
                users_service
                    .resend_verification_link(email)
                    .map_err(ControllerError::from),
            ),

            // POST /email_verify/apply/<token>
            (&Post, Some(Route::EmailVerifyApply(token))) => serialize_future(
                users_service
                    .verify_email(token)
                    .map_err(ControllerError::from),
            ),

            // Fallback
            _ => Box::new(future::err(ControllerError::NotFound)),
        }
    }
}
