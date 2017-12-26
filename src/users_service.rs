use std::sync::Arc;

use futures::future;
use futures::Future;
use serde_json;
use validator::Validate;

use common::{TheError, TheFuture, TheRequest, TheResponse, MAX_USER_COUNT};
use error::Error as ApiError;
use error::StatusMessage;
use http_utils::*;
use payloads::{NewUser, UpdateUser};
use service::Service;
use users_repo::UsersRepo;

pub struct UsersService {
    pub users_repo: Arc<UsersRepo>
}

impl Service for UsersService {}

impl UsersService {
    pub fn find(&self, user_id: i32) -> TheFuture {
        let result = self.users_repo.find(user_id)
            .map_err(|e| ApiError::from(e))
            .and_then(|user| {
                serde_json::to_string(&user)
                    .map_err(|e| ApiError::from(e))
            });

        self.respond_with(result)
    }

    pub fn list(&self, req: TheRequest) -> TheFuture {
        let result: Result<String, ApiError> = req.uri().query()
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
            })
            .and_then(|(from, count)| {
                // Get users
                self.users_repo.list(from, count)
                    .map_err(|e| ApiError::from(e))
                    .and_then(|users| {
                        serde_json::to_string(&users)
                            .map_err(|e| ApiError::from(e))
                    })
            });

        self.respond_with(result)
    }

    pub fn create(&self, req: TheRequest) -> Box<Future<Item = TheResponse, Error = TheError>> {
        let users_repo = self.users_repo.clone();

        let result = read_body(req).and_then(move |body| {
            let result: Result<String, ApiError> = serde_json::from_slice::<NewUser>(&body.as_bytes())
                .map_err(|e| ApiError::from(e))
                .and_then(|payload| {
                    // General validation
                    match payload.validate() {
                        Ok(_) => Ok(payload),
                        Err(e) => Err(ApiError::from(e))
                    }
                })
                .and_then(|payload| {
                    // Unique e-mail validation
                    match users_repo.email_exists(payload.email.to_string()) {
                        Ok(false) => Ok(payload),
                        Ok(true) => Err(ApiError::BadRequest("E-mail already registered".to_string())),
                        Err(e) => Err(ApiError::from(e))
                    }
                })
                .and_then(|payload| {
                    // User creation
                    users_repo.create(payload)
                        .map_err(|e| ApiError::from(e))
                        .and_then(|user| {
                            serde_json::to_string(&user)
                                .map_err(|e| ApiError::from(e))
                        })
                });

            match result {
                Ok(data) => future::ok(response_with_json(data)),
                Err(err) => future::ok(response_with_error(ApiError::from(err)))
            }
        });

        Box::new(result)
    }

    pub fn update(&self, req: TheRequest, user_id: i32) -> Box<Future<Item = TheResponse, Error = TheError>> {
        let users_repo = self.users_repo.clone();

        let result = read_body(req).and_then(move |body| {
            let inner = users_repo.find(user_id)
                .map_err(|e| ApiError::from(e))
                .and_then(|_user| {
                    // TODO: from_string?
                    serde_json::from_slice::<UpdateUser>(&body.as_bytes())
                        .map_err(|e| ApiError::from(e))
                })
                .and_then(|payload| {
                    users_repo.update(user_id, &payload)
                        .map_err(|e| ApiError::from(e))
                        .and_then(|user| {
                            serde_json::to_string(&user)
                                .map_err(|e| ApiError::from(e))
                        })
                });

            match inner {
                Ok(data) => future::ok(response_with_json(data)),
                Err(err) => future::ok(response_with_error(ApiError::from(err)))
            }
        });

        Box::new(result)
    }

    pub fn deactivate(&self, user_id: i32) -> TheFuture {
        let result = self.users_repo.deactivate(user_id)
            .map_err(|e| ApiError::from(e))
            .and_then(|_user| {
                let message = StatusMessage::new("User has been deactivated");
                serde_json::to_string(&message)
                    .map_err(|e| ApiError::from(e))
            });

        self.respond_with(result)
    }
}
