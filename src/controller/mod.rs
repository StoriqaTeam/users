//! `Controller` is a top layer that handles all http-related
//! stuff like reading bodies, parsing params, forming a response.
//! Basically it provides inputs to `Service` layer and converts outputs
//! of `Service` layer to http responses

pub mod routes;
pub mod utils;

use std::str::FromStr;
use std::sync::Arc;

use chrono::Utc;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future;
use futures::Future;
use futures::IntoFuture;
use futures_cpupool::CpuPool;
use hyper::header::Authorization;
use hyper::server::Request;
use hyper::{Delete, Get, Post, Put};
use r2d2::{ManageConnection, Pool};
use validator::Validate;

use stq_http::client::ClientHandle;
use stq_http::controller::Controller;
use stq_http::controller::ControllerFuture;
use stq_http::request_util::parse_body;
use stq_http::request_util::serialize_future;
use stq_router::RouteParser;

use self::routes::Route;
use config::Config;
use errors::Error;
use models;
use repos::acl::RolesCacheImpl;
use repos::repo_factory::*;
use services::jwt::{JWTService, JWTServiceImpl};
use services::system::{SystemService, SystemServiceImpl};
use services::user_delivery_address::{UserDeliveryAddressService, UserDeliveryAddressServiceImpl};
use services::user_roles::{UserRolesService, UserRolesServiceImpl};
use services::users::{UsersService, UsersServiceImpl};

/// Controller handles route parsing and calling `Service` layer
pub struct ControllerImpl<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub route_parser: Arc<RouteParser<Route>>,
    pub config: Config,
    pub client_handle: ClientHandle,
    pub roles_cache: RolesCacheImpl,
    pub repo_factory: F,
    pub jwt_private_key: Vec<u8>,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ControllerImpl<T, M, F>
{
    /// Create a new controller based on services
    pub fn new(
        db_pool: Pool<M>,
        cpu_pool: CpuPool,
        client_handle: ClientHandle,
        config: Config,
        roles_cache: RolesCacheImpl,
        repo_factory: F,
        jwt_private_key: Vec<u8>,
    ) -> Self {
        let route_parser = Arc::new(routes::create_route_parser());
        Self {
            route_parser,
            db_pool,
            cpu_pool,
            client_handle,
            config,
            roles_cache,
            repo_factory,
            jwt_private_key,
        }
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > Controller for ControllerImpl<T, M, F>
{
    /// Handle a request and get future response
    fn call(&self, req: Request) -> ControllerFuture {
        let headers = req.headers().clone();
        let auth_header = headers.get::<Authorization<String>>();
        let user_id = auth_header.map(move |auth| auth.0.clone()).and_then(|id| i32::from_str(&id).ok());

        let cached_roles = self.roles_cache.clone();
        let system_service = SystemServiceImpl::default();
        let users_service = UsersServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            self.client_handle.clone(),
            user_id,
            self.repo_factory.clone(),
        );
        let jwt_service = JWTServiceImpl::new(
            self.db_pool.clone(),
            self.cpu_pool.clone(),
            self.client_handle.clone(),
            self.config.clone(),
            self.repo_factory.clone(),
            self.jwt_private_key.clone(),
        );
        let user_roles_service =
            UserRolesServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), cached_roles, self.repo_factory.clone());
        let user_delivery_address_service =
            UserDeliveryAddressServiceImpl::new(self.db_pool.clone(), self.cpu_pool.clone(), user_id, self.repo_factory.clone());

        let path = req.path().to_string();

        match (&req.method().clone(), self.route_parser.test(req.path())) {
            // GET /healthcheck
            (&Get, Some(Route::Healthcheck)) => {
                trace!("Received healthcheck request");
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

            // GET /users/by_email
            (&Get, Some(Route::UserByEmail)) => {
                if let Some(email) = parse_query!(req.query().unwrap_or_default(), "email" => String) {
                    debug!("Received request to get user by email {}", email);
                    serialize_future(users_service.find_by_email(email))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // GET /users/by_email failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // GET /users
            (&Get, Some(Route::Users)) => {
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => i32, "count" => i64) {
                    debug!("Received request to get {} users starting from {}", count, offset);
                    serialize_future(users_service.list(offset, count))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // GET /users failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /users
            (&Post, Some(Route::Users)) => serialize_future(
                parse_body::<models::SagaCreateProfile>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /users in SagaCreateProfile failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .and_then(move |payload| {
                        debug!("Received request to create profile: {:?}", &payload);
                        payload
                            .identity
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation of SagaCreateProfile failed!")
                                    .context(Error::Validate(e))
                                    .into()
                            })
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

                                let user = payload.user.map(|mut user| {
                                    user.email = user.email.to_lowercase();
                                    user
                                });

                                users_service.create(checked_new_ident, user)
                            })
                    }),
            ),

            // PUT /users/<user_id>
            (&Put, Some(Route::User(user_id))) => serialize_future(
                parse_body::<models::user::UpdateUser>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // PUT /users/<user_id> in UpdateUser failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .inspect(|payload| {
                        debug!("Received request to update user: {:?}", &payload);
                    })
                    .and_then(move |update_user| {
                        update_user
                            .validate()
                            .map_err(|e| format_err!("Validation of UpdateUser failed!").context(Error::Validate(e)).into())
                            .into_future()
                            .inspect(|_| {
                                debug!("Validation success");
                            })
                            .and_then(move |_| users_service.update(user_id, update_user))
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
                    .map_err(|e| {
                        e.context("Parsing body // POST /jwt/email in NewEmailIdentity failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .and_then(move |new_ident| {
                        debug!("Received request to authenticate with email: {}", &new_ident);
                        new_ident
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation of NewEmailIdentity failed!")
                                    .context(Error::Validate(e))
                                    .into()
                            })
                            .into_future()
                            .inspect(|_| {
                                debug!("Validation success");
                            })
                            .and_then(move |_| {
                                let checked_new_ident = models::identity::NewEmailIdentity {
                                    email: new_ident.email.to_lowercase(),
                                    password: new_ident.password,
                                };

                                let now = Utc::now().timestamp();

                                jwt_service.create_token_email(checked_new_ident, now)
                            })
                    }),
            ),

            // POST /jwt/google
            (&Post, Some(Route::JWTGoogle)) => serialize_future(
                parse_body::<models::jwt::ProviderOauth>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /jwt/google in ProviderOauth failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .inspect(|payload| {
                        debug!("Received request to authenticate with Google token: {:?}", &payload);
                    })
                    .and_then(move |oauth| {
                        let now = Utc::now().timestamp();

                        jwt_service.create_token_google(oauth, now)
                    }),
            ),

            // POST /jwt/facebook
            (&Post, Some(Route::JWTFacebook)) => serialize_future(
                parse_body::<models::jwt::ProviderOauth>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /jwt/facebook in ProviderOauth failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .inspect(|payload| {
                        debug!("Received request to authenticate with Facebook token: {:?}", &payload);
                    })
                    .and_then(move |oauth| {
                        let now = Utc::now().timestamp();

                        jwt_service.create_token_facebook(oauth, now)
                    }),
            ),

            // GET /user_roles/<user_id>
            (&Get, Some(Route::UserRole(user_id))) => {
                debug!("Received request to get roles for user {}", user_id);
                serialize_future(user_roles_service.get_roles(user_id))
            }

            // POST /user_roles
            (&Post, Some(Route::UserRoles)) => serialize_future(
                parse_body::<models::NewUserRole>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /user_roles in NewUserRole failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .inspect(|payload| {
                        debug!("Received request to create role: {:?}", payload);
                    })
                    .and_then(move |new_role| user_roles_service.create(new_role)),
            ),

            // DELETE /user_roles/<user_role_id>
            (&Delete, Some(Route::UserRoles)) => serialize_future(
                parse_body::<models::OldUserRole>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // DELETE /user_roles/<user_role_id> in OldUserRole failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .inspect(|payload| {
                        debug!("Received request to remove role: {:?}", payload);
                    })
                    .and_then(move |old_role| user_roles_service.delete(old_role)),
            ),

            // POST /roles/default/<user_id>
            (&Post, Some(Route::DefaultRole(user_id))) => {
                debug!("Received request to add default role for user {}", user_id);
                serialize_future(user_roles_service.create_default(user_id.0))
            }

            // DELETE /roles/default/<user_id>
            (&Delete, Some(Route::DefaultRole(user_id))) => {
                debug!("Received request to delete default role for user {}", user_id);
                serialize_future(user_roles_service.delete_default(user_id.0))
            }

            // POST /users/password_change
            (&Post, Some(Route::PasswordChange)) => serialize_future(
                parse_body::<models::ChangeIdentityPassword>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /users/password_change in ChangeIdentityPassword failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .inspect(|payload| {
                        debug!("Received request to start password change: {:?}", payload);
                    })
                    .and_then(move |change_req| {
                        change_req
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation of ChangeIdentityPassword failed!")
                                    .context(Error::Validate(e))
                                    .into()
                            })
                            .into_future()
                            .and_then(move |_| users_service.change_password(change_req))
                    }),
            ),

            // Post /users/password_reset_token
            (&Post, Some(Route::UserPasswordResetToken)) => serialize_future(
                parse_body::<models::ResetRequest>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // Post /users/password_reset_token in ResetRequest failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .inspect(|payload| {
                        debug!("Received request to start password reset: {:?}", payload);
                    })
                    .and_then(move |reset_req| {
                        reset_req
                            .validate()
                            .map_err(|e| format_err!("Validation of ResetRequest failed!").context(Error::Validate(e)).into())
                            .into_future()
                            .and_then(move |_| users_service.get_password_reset_token(reset_req.email))
                    }),
            ),

            // PUT /users/password_reset_token
            (&Put, Some(Route::UserPasswordResetToken)) => serialize_future(
                parse_body::<models::ResetApply>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // PUT /users/password_reset_token in ResetApply failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .inspect(|payload| {
                        debug!("Received request to complete password reset: {}", payload);
                    })
                    .and_then(move |reset_apply| {
                        reset_apply
                            .validate()
                            .map_err(|e| format_err!("Validation of ResetApply failed!").context(Error::Validate(e)).into())
                            .into_future()
                            .and_then(move |_| users_service.password_reset_apply(reset_apply.token, reset_apply.password))
                    }),
            ),

            // Post /users/email_verify_token
            (&Post, Some(Route::UserEmailVerifyToken)) => serialize_future(
                parse_body::<models::ResetRequest>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // Post /users/email_verify_token in ResetRequest failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .inspect(|payload| {
                        debug!("Received request to get user with email {} verify token", payload.email);
                    })
                    .and_then(move |reset_req| {
                        reset_req
                            .validate()
                            .map_err(|e| format_err!("Validation of ResetRequest failed!").context(Error::Validate(e)).into())
                            .into_future()
                            .and_then(move |_| users_service.get_email_verification_token(reset_req.email))
                    }),
            ),

            // Put /users/email_verify_token
            (&Put, Some(Route::UserEmailVerifyToken)) => {
                if let Some(token) = parse_query!(req.query().unwrap_or_default(), "token" => String) {
                    debug!("Received request to apply token {} to verify email.", token);
                    serialize_future(users_service.verify_email(token))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // Put /users/email_verify_token failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // GET /users/delivery_addresses/<user_id>
            (&Get, Some(Route::UserDeliveryAddress(user_id))) => {
                debug!("Received request to get addresses for user {}", user_id);
                serialize_future(user_delivery_address_service.get_addresses(user_id))
            }

            // POST /users/delivery_addresses
            (&Post, Some(Route::UserDeliveryAddresses)) => serialize_future(
                parse_body::<models::NewUserDeliveryAddress>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /users/delivery_addresses in NewUserDeliveryAddress failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .inspect(|payload| {
                        debug!("Received request to create delivery address: {:?}", payload);
                    })
                    .and_then(move |new_address| {
                        new_address
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation of NewUserDeliveryAddress failed!")
                                    .context(Error::Validate(e))
                                    .into()
                            })
                            .into_future()
                            .and_then(move |_| user_delivery_address_service.create(new_address))
                    }),
            ),

            // PUT /users/delivery_addresses/<id>
            (&Put, Some(Route::UserDeliveryAddress(id))) => serialize_future(
                parse_body::<models::UpdateUserDeliveryAddress>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body PUT /users/delivery_addresses/<id> in UpdateUserDeliveryAddress failed!")
                            .context(Error::Parse)
                            .into()
                    })
                    .inspect(|payload| {
                        debug!("Received request to update delivery address: {:?}", payload);
                    })
                    .and_then(move |new_address| {
                        new_address
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation of UpdateUserDeliveryAddress failed!")
                                    .context(Error::Validate(e))
                                    .into()
                            })
                            .into_future()
                            .and_then(move |_| user_delivery_address_service.update(id, new_address))
                    }),
            ),

            // DELETE /users/delivery_addresses/<id>
            (&Delete, Some(Route::UserDeliveryAddress(id))) => {
                debug!("Received request to delete user delivery address with id {}", id);
                serialize_future(user_delivery_address_service.delete(id))
            }

            // Fallback
            (m, _) => Box::new(future::err(
                format_err!("Request to non existing endpoint in users microservice! {:?} {:?}", m, path)
                    .context(Error::NotFound)
                    .into(),
            )),
        }
    }
}
