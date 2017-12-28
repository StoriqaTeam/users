use std::sync::Arc;

use futures::future;
use futures::Future;
use serde_json;
use validator::Validate;

use common::{TheError, TheResponse, TheFuture, TheRequest, MAX_USER_COUNT};
use error::Error as ApiError;
use payloads::user::{NewUser, UpdateUser};
use repos::users::UsersRepo;
use responses::status::StatusMessage;
use services::Service;
use utils::http::*;

/// Users services, responsible for User-related CRUD operations
pub struct UsersService {
    pub users_repo: Arc<UsersRepo>
}

impl Service for UsersService {}

impl UsersService {
    /// Returns user by ID
    pub fn get(&self, user_id: i32) -> TheFuture {
        let future = self.users_repo.find(user_id)
            .and_then(|user| {
                serde_json::to_string(&user).map_err(|e| ApiError::from(e))
            })
            .then(|res| match res {
                Ok(data) => future::ok(response_with_json(data)),
                Err(err) => future::ok(response_with_error(err))
            });

        Box::new(future)
    }

    /// Returns list of users, limited by `from` and `count` request parameters
    pub fn list(&self, req: TheRequest) -> TheFuture {
        // TODO - Move request parsing to separate layer
        let request = req.uri().query()
            .ok_or(ApiError::BadRequest("Missing query parameters: `from`, `count`".to_string()))
            .and_then(|query| Ok(query_params(query)))
            .and_then(|params| {
                // Extract `from` param
                Ok((params.clone(), params.get("from").and_then(|v| v.parse::<i32>().ok())
                    .ok_or(ApiError::BadRequest("Invalid value provided for `from`".to_string()))))
            })
            .and_then(|(params, from)| {
                // Extract `count` param
                Ok((from, params.get("count").and_then(|v| v.parse::<i64>().ok())
                    .ok_or(ApiError::BadRequest("Invalid value provided for `count`".to_string()))))
            })
            .and_then(|(from, count)| {
                // Transform tuple of `Result`s to `Result` of tuple
                match (from, count) {
                    (Ok(x), Ok(y)) if x > 0 && y < MAX_USER_COUNT => Ok((x, y)),
                    (_, _) => Err(ApiError::BadRequest("Invalid values provided for `from` or `count`".to_string())),
                }
            });

        match request {
            Ok((from, count)) => {
                let result = self.users_repo.list(from, count)
                    .and_then(|user| {
                        serde_json::to_string(&user).map_err(|e| ApiError::from(e))
                    })
                    .then(|res| match res {
                        Ok(data) => future::ok(response_with_json(data)),
                        Err(err) => future::ok(response_with_error(err))
                    });

                Box::new(result)
            },
            Err(err) => self.respond_with(Err(err))
        }
    }

    /// Creates user from payload, provided in request body
    pub fn create(&self, req: TheRequest) -> TheFuture {
        let users_repo = self.users_repo.clone();

        let future = read_body(req).and_then(move |body| {
            let result = serde_json::from_str::<NewUser>(&body)
                .map_err(|e| ApiError::from(e))
                .and_then(|payload| match payload.validate() {
                    Ok(_) => Ok(payload),
                    Err(e) => Err(ApiError::from(e))
                });

            match result {
                Ok(payload) => {
                    let inner = users_repo.email_exists(payload.email.to_string())
                        .and_then(|res| match res {
                            false => future::ok(payload),
                            true => future::err(ApiError::BadRequest("E-mail already registered".to_string()))
                        })
                        .and_then(move |user| {
                            users_repo.create(user)
                        })
                        .and_then(|user| {
                            serde_json::to_string(&user).map_err(|e| ApiError::from(e))
                        })
                        .then(|res| match res {
                            Ok(data) => future::ok(response_with_json(data)),
                            Err(err) => future::ok(response_with_error(err))
                        });

                    Box::new(inner)
                },
                Err(err) => unimplemented!()
            }
        });

        Box::new(future)
    }

    /// Updates specific user from payload, provided in request body
    /*
    pub fn update(&self, req: TheRequest, user_id: i32) -> TheFuture {
        let users_repo = self.users_repo.clone();

        let result = read_body(req).and_then(move |body| {
            let inner = users_repo.find(user_id)
                .map_err(|e| ApiError::from(e))
                .and_then(|_user| {
                    serde_json::from_str::<UpdateUser>(&body).map_err(|e| ApiError::from(e))
                })
                .and_then(|payload| {
                    users_repo.update(user_id, &payload)
                        .map_err(|e| ApiError::from(e))
                        .and_then(|user| {
                            serde_json::to_string(&user).map_err(|e| ApiError::from(e))
                        })
                });

            match inner {
                Ok(data) => future::ok(response_with_json(data)),
                Err(err) => future::ok(response_with_error(ApiError::from(err)))
            }
        });

        Box::new(result)
    }
    */

    /// Deactivates specific user
    pub fn deactivate(&self, user_id: i32) -> TheFuture {
        let future = self.users_repo.deactivate(user_id)
            .map_err(|e| ApiError::from(e))
            .and_then(|_user| {
                let message = StatusMessage::new("User has been deactivated");
                serde_json::to_string(&message)
                    .map_err(|e| ApiError::from(e))
            })
            .then(|res| match res {
                Ok(data) => future::ok(response_with_json(data)),
                Err(err) => future::ok(response_with_error(err))
            });

        Box::new(future)
    }
}
