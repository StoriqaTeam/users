use std::sync::Arc;

use futures::future;
use futures::Future;

use error::Error as ApiError;
use models::user::User;
use models::jwt::JWT;
use payloads::user::{NewUser, UpdateUser};
use payloads::jwt::ProviderOauth;
use repos::users::UsersRepo;
use repos::jwt::JWTRepo;

/// Users services, responsible for User-related CRUD operations
pub struct UsersService {
    pub users_repo: Arc<UsersRepo>,
    pub jwt_repo: Arc<JWTRepo>,
}

impl UsersService {
    /// Returns user by ID
    pub fn get(&self, user_id: i32) -> Box<Future<Item = User, Error = ApiError>> {
        Box::new(self.users_repo.find(user_id))
    }

    /// Lists users limited by `from` and `count` parameters
    pub fn list(&self, from: i32, count: i64) -> Box<Future<Item = Vec<User>, Error = ApiError>> {
        Box::new(self.users_repo.list(from, count))
    }

    /// Deactivates specific user
    pub fn deactivate(&self, user_id: i32) -> Box<Future<Item = User, Error = ApiError>> {
        Box::new(self.users_repo.deactivate(user_id))
    }

    /// Creates new user
    pub fn create(&self, payload: NewUser) -> Box<Future<Item = User, Error = ApiError>> {
        let insert_repo = self.users_repo.clone();

        let future = self.users_repo.email_exists(payload.email.to_string())
            .map(|exists| (payload, exists))
            .and_then(|(payload, exists)| match exists {
                false => future::ok(payload),
                true => future::err(ApiError::BadRequest("E-mail already registered".to_string()))
            })
            .and_then(move |user| {
                insert_repo.create(user)
            });

        Box::new(future)
    }

    /// Updates specific user
    pub fn update(&self, user_id: i32, payload: UpdateUser) -> Box<Future<Item = User, Error = ApiError>> {
        let update_repo = self.users_repo.clone();

        let future = self.users_repo.find(user_id)
            .and_then(move |_user| update_repo.update(user_id, payload));

        Box::new(future)
    }

    /// Creates new JWT token by email
    pub fn create_token_email(&self, user: NewUser) -> Box<Future<Item = JWT, Error = ApiError>> {
        let insert_repo = self.users_repo.clone();
        let jwt_repo = self.jwt_repo.clone();

        let future = self.users_repo.email_exists(user.email.to_string())
            .map(|exists| (user, exists))
            .and_then(|(user, exists)| match exists {
                true => future::ok(user),
                false => insert_repo.create(user).map(|_| user),
            })
            .and_then(|user| future::ok(jwt_repo.create_token_user(user)));

        Box::new(future)
    }

    /// Creates new JWT token by google
    pub fn create_token_google(&self, payload: ProviderOauth) -> Box<Future<Item = JWT, Error = ApiError>> {
        let jwt_repo = self.jwt_repo.clone();
        let future = future::ok(jwt_repo.create_token_google(payload));
        Box::new(future)
    }

    /// Creates new JWT token by facebook
    pub fn create_token_facebook(&self, payload: ProviderOauth) -> Box<Future<Item = JWT, Error = ApiError>> {
        let jwt_repo = self.jwt_repo.clone();
        let future = future::ok(jwt_repo.create_token_facebook(payload));
        Box::new(future)
    }
}
