use std::sync::Arc;
use std::string::ToString;

use futures::Future;
use futures::future;
use hyper::{Get, Post, Put, Delete};
use hyper::server::{Service, Request};

use common::{TheError, TheFuture, TheRequest, TheResponse};
use error::Error as ApiError;
use services::system::SystemService;
use services::users::UsersService;
use router::{Route, Router};
use utils::http::{response_with_error, response_with_json, parse_body};
use serde_json;

use payloads;

macro_rules! serialize_future {
    ($e:expr) => (Box::new($e.and_then(|resp| serde_json::to_string(&resp).map_err(|e| ApiError::from(e)))))
}

/// Application contains all facades, services and `Router`
pub struct Application {
    pub router: Arc<Router>,
    pub system_service: Arc<SystemService>,
    pub users_service: Arc<UsersService>,
}

impl Service for Application {
    type Request = TheRequest;
    type Response = TheResponse;
    type Error = TheError;
    type Future = TheFuture;

    fn call(&self, req: Request) -> Box<Future<Item = TheResponse, Error = TheError>> {
        info!("{:?}", req);

        // match (req.method(), self.router.test(req.path())) {
        //     // GET /healthcheck
        //     (&Get, Some(Route::Healthcheck)) => self.system_service.healthcheck(),
        //     // GET /users/<user_id>
        //     (&Get, Some(Route::User(user_id))) => self.users_facade.get(user_id),
        //     // GET /users
        //     (&Get, Some(Route::Users)) => self.users_facade.list(req),
        //     // POST /users
        //     (&Post, Some(Route::Users)) => self.users_facade.create(req),
        //     // PUT /users/<user_id>
        //     (&Put, Some(Route::User(user_id))) => self.users_facade.update(req, user_id),
        //     // DELETE /users/<user_id>
        //     (&Delete, Some(Route::User(user_id))) => self.users_facade.deactivate(user_id),
        //     // Fallback
        //     _ => Box::new(future::ok(response_with_error(ApiError::NotFound)))
        // }
        Box::new(
            self.call_service(req).then(|res| match res {
                Ok(data) => future::ok(response_with_json(data)),
                Err(err) => future::ok(response_with_error(err))
            })
        )
        // Box::new(future::ok(response_with_error(ApiError::NotFound)))
    }
}

impl Application {
    fn call_service(&self, req: Request) -> Box<Future<Item = String, Error = ApiError>>
    {
        match (req.method(), self.router.test(req.path())) {
            // GET /healthcheck
            (&Get, Some(Route::Healthcheck)) => serialize_future!(self.system_service.healthcheck()),

            // GET /users/<user_id>
            (&Get, Some(Route::User(user_id))) => serialize_future!(self.users_service.get(user_id)),

            // GET /users
            (&Get, Some(Route::Users)) => {
                if let (Some(from), Some(to)) = params!(req.query().unwrap_or_default(), "from" -> i32, "to" -> i64) {
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
                        .and_then(move |new_user| users_service.create(new_user))
                )
            },
            // PUT /users/<user_id>
            // (&Put, Some(Route::User(user_id))) => self.users_facade.update(req, user_id),
            // DELETE /users/<user_id>
            // (&Delete, Some(Route::User(user_id))) => self.users_facade.deactivate(user_id),
            // Fallback
            _ => Box::new(future::err(ApiError::NotFound))
        }

    }

}
