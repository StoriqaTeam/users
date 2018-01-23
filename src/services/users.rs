use std::sync::Arc;

use futures::future;
use futures::Future;

use models::user::{User, NewUser, UpdateUser};
use repos::users::UsersRepo;
use super::types::ServiceFuture;
use super::error::Error;


pub trait UsersService {
    /// Returns user by ID
    fn get(&self, user_id: i32) -> ServiceFuture<User>;
    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<User>>;
    /// Deactivates specific user
    fn deactivate(&self, user_id: i32) -> ServiceFuture<User>;
    /// Creates new user
    fn create(&self, payload: NewUser) -> ServiceFuture<User>;
    /// Updates specific user
    fn update(&self, user_id: i32, payload: UpdateUser) -> ServiceFuture<User>;
}

/// Users services, responsible for User-related CRUD operations
pub struct UsersServiceImpl<U: 'static + UsersRepo> {
    pub users_repo: Arc<U>,
}

impl<U: UsersRepo> UsersService for UsersServiceImpl<U> {
    /// Returns user by ID
    fn get(&self, user_id: i32) -> ServiceFuture<User> {
        Box::new(self.users_repo.find(user_id).map_err(|e| Error::from(e)))
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<User>> {
        Box::new(
            self.users_repo
                .list(from, count)
                .map_err(|e| Error::from(e)),
        )
    }

    /// Deactivates specific user
    fn deactivate(&self, user_id: i32) -> ServiceFuture<User> {
        Box::new(
            self.users_repo
                .deactivate(user_id)
                .map_err(|e| Error::from(e)),
        )
    }

    /// Creates new user
    fn create(&self, payload: NewUser) -> ServiceFuture<User> {
        let users_repo = self.users_repo.clone();


        Box::new(
            users_repo
                .email_exists(payload.user_email.to_string())
                .map(|exists| (payload, exists))
                .map_err(Error::from)
                .and_then(|(payload, exists)| match exists {
                    false => future::ok(payload),
                    true => future::err(Error::Validate(validation_errors!({"email": ["email" => "Email already exists"]})))
                })
                .and_then(move |user| {
                    users_repo.create(user).map_err(|e| Error::from(e))
                }),
        )
    }

    /// Updates specific user
    fn update(&self, user_id: i32, payload: UpdateUser) -> ServiceFuture<User> {
        let update_repo = self.users_repo.clone();

        Box::new(
            update_repo
                .find(user_id)
                .and_then(move |_user| update_repo.update(user_id, payload))
                .map_err(|e| Error::from(e)),
        )
    }
}
