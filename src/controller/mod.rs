//! `Controller` is a top layer that handles all http-related
//! stuff like reading bodies, parsing params, forming a response.
//! Basically it provides inputs to `Service` layer and converts outputs
//! of `Service` layer to http responses

pub mod context;
pub mod routes;
pub mod utils;

use std::str::FromStr;

use chrono::Utc;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::future;
use futures::Future;
use futures::IntoFuture;
use hyper::header::Authorization;
use hyper::server::Request;
use hyper::{Delete, Get, Post, Put};
use r2d2::ManageConnection;
use validator::Validate;

use stq_http::controller::Controller;
use stq_http::controller::ControllerFuture;
use stq_http::request_util::parse_body;
use stq_http::request_util::serialize_future;
use stq_types::UserId;

use self::context::{DynamicContext, StaticContext};
use self::routes::Route;
use errors::Error;
use models;
use repos::repo_factory::*;
use services::jwt::JWTService;
use services::user_roles::UserRolesService;
use services::users::UsersService;
use services::Service;

/// Controller handles route parsing and calling `Service` layer
pub struct ControllerImpl<T, M, F>
where
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
{
    pub static_context: StaticContext<T, M, F>,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > ControllerImpl<T, M, F>
{
    /// Create a new controller based on services
    pub fn new(static_context: StaticContext<T, M, F>) -> Self {
        Self { static_context }
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
        let user_id = auth_header
            .map(|auth| auth.0.clone())
            .and_then(|id| i32::from_str(&id).ok())
            .map(UserId);
        let dynamic_context = DynamicContext::new(user_id);

        let service = Service::new(self.static_context.clone(), dynamic_context);

        let path = req.path().to_string();

        match (&req.method().clone(), self.static_context.route_parser.test(req.path())) {
            // GET /users/<user_id>
            (&Get, Some(Route::User(user_id))) => {
                debug!("Received request to get user info for ID {}", user_id);
                serialize_future(service.get(user_id))
            }

            // GET /users/current
            (&Get, Some(Route::Current)) => {
                debug!("Received request to get current user info.");
                serialize_future(service.current())
            }

            // GET /users/by_email
            (&Get, Some(Route::UserByEmail)) => {
                if let Some(email) = parse_query!(req.query().unwrap_or_default(), "email" => String) {
                    debug!("Received request to get user by email {}", email);
                    serialize_future(service.find_by_email(email.to_lowercase()))
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
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => UserId, "count" => i64) {
                    debug!("Received request to get {} users starting from {}", count, offset);
                    serialize_future(service.list(offset, count))
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
                    }).and_then(move |payload| {
                        debug!("Received request to create profile: {:?}", &payload);
                        payload
                            .identity
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation of SagaCreateProfile failed!")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .inspect(|_| {
                                debug!("Validation success");
                            }).and_then(move |_| {
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

                                service.create(checked_new_ident, user)
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
                    }).inspect(|payload| {
                        debug!("Received request to update user: {:?}", &payload);
                    }).and_then(move |update_user| {
                        update_user
                            .validate()
                            .map_err(|e| format_err!("Validation of UpdateUser failed!").context(Error::Validate(e)).into())
                            .into_future()
                            .inspect(|_| {
                                debug!("Validation success");
                            }).and_then(move |_| service.update(user_id, update_user))
                    }),
            ),

            // POST /users/<user_id>/block
            (&Post, Some(Route::UserBlock(user_id))) => {
                debug!("Received request to block user {}", user_id);
                serialize_future(service.set_block_status(user_id, true))
            }

            // POST /users/<user_id>/unblock
            (&Post, Some(Route::UserUnblock(user_id))) => {
                debug!("Received request to unblock user {}", user_id);
                serialize_future(service.set_block_status(user_id, false))
            }

            // DELETE /users/<user_id>
            (&Delete, Some(Route::User(user_id))) => {
                debug!("Received request to deactivate user {}", user_id);
                serialize_future(service.deactivate(user_id))
            }

            // DELETE /user_by_saga_id/<user_id>
            (&Delete, Some(Route::UserBySagaId(saga_id))) => {
                debug!("Received request to delete user with saga ID {}", saga_id);
                serialize_future(service.delete_by_saga_id(saga_id))
            }

            // POST /jwt/email
            (&Post, Some(Route::JWTEmail)) => serialize_future(
                parse_body::<models::identity::NewEmailIdentity>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /jwt/email in NewEmailIdentity failed!")
                            .context(Error::Parse)
                            .into()
                    }).and_then(move |new_ident| {
                        debug!("Received request to authenticate with email: {}", &new_ident);
                        new_ident
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation of NewEmailIdentity failed!")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .inspect(|_| {
                                debug!("Validation success");
                            }).and_then(move |_| {
                                let checked_new_ident = models::identity::NewEmailIdentity {
                                    email: new_ident.email.to_lowercase(),
                                    password: new_ident.password,
                                };

                                let now = Utc::now().timestamp();

                                service.create_token_email(checked_new_ident, now)
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
                    }).inspect(|payload| {
                        debug!("Received request to authenticate with Google token: {:?}", &payload);
                    }).and_then(move |oauth| {
                        let now = Utc::now().timestamp();

                        service.create_token_google(oauth, now)
                    }),
            ),

            // POST /jwt/facebook
            (&Post, Some(Route::JWTFacebook)) => serialize_future(
                parse_body::<models::jwt::ProviderOauth>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /jwt/facebook in ProviderOauth failed!")
                            .context(Error::Parse)
                            .into()
                    }).inspect(|payload| {
                        debug!("Received request to authenticate with Facebook token: {:?}", &payload);
                    }).and_then(move |oauth| {
                        let now = Utc::now().timestamp();

                        service.create_token_facebook(oauth, now)
                    }),
            ),

            (Get, Some(Route::RolesByUserId { user_id })) => {
                debug!("Received request to get roles by user id {}", user_id);
                serialize_future({ service.get_roles(user_id) })
            }
            (Post, Some(Route::Roles)) => serialize_future({
                parse_body::<models::NewUserRole>(req.body()).and_then(move |data| {
                    debug!("Received request to create role {:?}", data);
                    service.create_user_role(data)
                })
            }),
            (Delete, Some(Route::RolesByUserId { user_id })) => {
                debug!("Received request to delete role by user id {}", user_id);
                serialize_future({ service.delete_user_role_by_user_id(user_id) })
            }
            (Delete, Some(Route::RoleById { id })) => {
                debug!("Received request to delete role by id {}", id);
                serialize_future({ service.delete_user_role_by_id(id) })
            }

            // POST /users/password_change
            (&Post, Some(Route::PasswordChange)) => serialize_future(
                parse_body::<models::ChangeIdentityPassword>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // POST /users/password_change in ChangeIdentityPassword failed!")
                            .context(Error::Parse)
                            .into()
                    }).inspect(|payload| {
                        debug!("Received request to start password change: {:?}", payload);
                    }).and_then(move |change_req| {
                        change_req
                            .validate()
                            .map_err(|e| {
                                format_err!("Validation of ChangeIdentityPassword failed!")
                                    .context(Error::Validate(e))
                                    .into()
                            }).into_future()
                            .and_then(move |_| service.change_password(change_req))
                    }),
            ),

            // Post /users/password_reset_token
            (&Post, Some(Route::UserPasswordResetToken)) => serialize_future(
                parse_body::<models::ResetRequest>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // Post /users/password_reset_token in ResetRequest failed!")
                            .context(Error::Parse)
                            .into()
                    }).inspect(|payload| {
                        debug!("Received request to start password reset: {:?}", payload);
                    }).and_then(move |reset_req| {
                        reset_req
                            .validate()
                            .map_err(|e| format_err!("Validation of ResetRequest failed!").context(Error::Validate(e)).into())
                            .into_future()
                            .and_then(move |_| service.get_password_reset_token(reset_req.email.to_lowercase()))
                    }),
            ),

            // PUT /users/password_reset_token
            (&Put, Some(Route::UserPasswordResetToken)) => serialize_future(
                parse_body::<models::ResetApply>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // PUT /users/password_reset_token in ResetApply failed!")
                            .context(Error::Parse)
                            .into()
                    }).inspect(|payload| {
                        debug!("Received request to complete password reset: {}", payload);
                    }).and_then(move |reset_apply| {
                        reset_apply
                            .validate()
                            .map_err(|e| format_err!("Validation of ResetApply failed!").context(Error::Validate(e)).into())
                            .into_future()
                            .and_then(move |_| service.password_reset_apply(reset_apply.token, reset_apply.password))
                    }),
            ),

            // Post /users/email_verify_token
            (&Post, Some(Route::UserEmailVerifyToken)) => serialize_future(
                parse_body::<models::ResetRequest>(req.body())
                    .map_err(|e| {
                        e.context("Parsing body // Post /users/email_verify_token in ResetRequest failed!")
                            .context(Error::Parse)
                            .into()
                    }).inspect(|payload| {
                        debug!("Received request to get user with email {} verify token", payload.email);
                    }).and_then(move |reset_req| {
                        reset_req
                            .validate()
                            .map_err(|e| format_err!("Validation of ResetRequest failed!").context(Error::Validate(e)).into())
                            .into_future()
                            .and_then(move |_| service.get_email_verification_token(reset_req.email.to_lowercase()))
                    }),
            ),

            // Put /users/email_verify_token
            (&Put, Some(Route::UserEmailVerifyToken)) => {
                if let Some(token) = parse_query!(req.query().unwrap_or_default(), "token" => String) {
                    debug!("Received request to apply token {} to verify email.", token);
                    serialize_future(service.verify_email(token))
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // Put /users/email_verify_token failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
            }

            // POST /users/search
            (&Post, Some(Route::UsersSearch)) => {
                if let (Some(offset), Some(count)) = parse_query!(req.query().unwrap_or_default(), "offset" => UserId, "count" => i64) {
                    debug!("Received request to search {} users starting from {}", count, offset);
                    serialize_future(
                        parse_body::<models::UsersSearchTerms>(req.body())
                            .map_err(|e| {
                                e.context("Parsing body // POST /users/search in UsersSearchTerms failed!")
                                    .context(Error::Parse)
                                    .into()
                            }).inspect(|payload| {
                                debug!("Received request to search for user whith payload {:?}", payload);
                            }).and_then(move |payload| service.search(offset, count, payload)),
                    )
                } else {
                    Box::new(future::err(
                        format_err!("Parsing query parameters // POST /users/search failed!")
                            .context(Error::Parse)
                            .into(),
                    ))
                }
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
