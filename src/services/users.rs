use std::sync::Arc;

use futures::future;
use futures::Future;

use error::Error as ApiError;
use models::user::User;
use payloads::user::{NewUser, UpdateUser};
use repos::users::UsersRepo;

/// Users services, responsible for User-related CRUD operations
pub struct UsersService {
    pub users_repo: Arc<UsersRepo>
}

impl UsersService {

    pub fn get(&self, user_id: i32) -> Box<Future<Item = User, Error = ApiError>> {
        Box::new(self.users_repo.find(user_id))
    }

    pub fn list(&self, from: i32, count: i64) -> Box<Future<Item = Vec<User>, Error = ApiError>> {
        Box::new(self.users_repo.list(from, count))
    }

    pub fn deactivate(&self, user_id: i32) -> Box<Future<Item = User, Error = ApiError>> {
        Box::new(self.users_repo.deactivate(user_id))
    }

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

    pub fn update(&self, user_id: i32, payload: UpdateUser) -> Box<Future<Item = User, Error = ApiError>> {
        let update_repo = self.users_repo.clone();

        let future = self.users_repo.find(user_id)
            .and_then(move |_user| update_repo.update(user_id, payload));

        Box::new(future)
    }
}
