use std::sync::Arc;

use futures::future;
use serde_json;

use common::TheFuture;
use http_utils::*;
use error::Error as ApiError;
use users_repo::UsersRepo;

pub struct UsersService {
    users_repo: Arc<UsersRepo>
}

impl UsersService {
    fn respond_with(&self, result: Result<String, ApiError>) -> TheFuture {
        match result {
            Ok(response) => Box::new(future::ok(response_with_json(response))),
            Err(err) => Box::new(future::ok(response_with_error(err)))
        }
    }

    pub fn find(&self, user_id: i32) -> TheFuture {
        let result = self.users_repo.find(user_id)
            .map_err(|e| ApiError::from(e))
            .and_then(|user| {
                serde_json::to_string(&user)
                    .map_err(|e| ApiError::from(e))
            });

        self.respond_with(result)
    }
}
