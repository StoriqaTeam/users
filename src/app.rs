use std::sync::Arc;

use futures::Future;
use futures::future;
use hyper::{Get, Post, Put, Delete};
use hyper::server::{Service, Request};

use common::{TheError, TheFuture, TheRequest, TheResponse};
use error::Error as ApiError;
use facades::system::SystemFacade;
use facades::users::UsersFacade;
use router::{Route, Router};
use utils::http::response_with_error;

/// Application contains all facades, services and `Router`
pub struct Application {
    pub router: Arc<Router>,
    pub system_facade: Arc<SystemFacade>,
    pub users_facade: Arc<UsersFacade>,
}

impl Service for Application {
    type Request = TheRequest;
    type Response = TheResponse;
    type Error = TheError;
    type Future = TheFuture;

    fn call(&self, req: Request) -> Box<Future<Item = TheResponse, Error = TheError>> {
        info!("{:?}", req);

        match (req.method(), self.router.test(req.path())) {
            // GET /healthcheck
            (&Get, Some(Route::Healthcheck)) => self.system_facade.healthcheck(),
            // GET /users/<user_id>
            (&Get, Some(Route::User(user_id))) => self.users_facade.get(user_id),
            // GET /users
            (&Get, Some(Route::Users)) => self.users_facade.list(req),
            // POST /users
            (&Post, Some(Route::Users)) => self.users_facade.create(req),
            // PUT /users/<user_id>
            (&Put, Some(Route::User(user_id))) => self.users_facade.update(req, user_id),
            // DELETE /users/<user_id>
            (&Delete, Some(Route::User(user_id))) => self.users_facade.deactivate(user_id),
            
            // POST /jwt/email
            (&Post, Some(Route::JWT_email)) => self.users_facade.create_token_by_email(req),
            // POST /jwt/google
            (&Post, Some(Route::JWT_google)) => self.users_facade.create_token_by_google(req),
            // POST /jwt/facebook
            (&Post, Some(Route::JWT_facebook)) => self.users_facade.create_token_by_facebook(req),

            // Fallback
            _ => Box::new(future::ok(response_with_error(ApiError::NotFound)))
        }
    }
}
