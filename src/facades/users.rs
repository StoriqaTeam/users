use std::sync::Arc;

use futures::future;
use futures::{IntoFuture, Future};
use serde_json;
use validator::Validate;

use common::{TheFuture, TheRequest, MAX_USER_COUNT};
use error::Error as ApiError;
use payloads::user::{NewUser, UpdateUser};
use payloads::jwt::ProviderOauth;
use responses::status::StatusMessage;
use services::users::UsersService;
use services::jwt::JWTService;
use utils::httpserver::*;

pub struct UsersFacade {
    pub users_service: Arc<UsersService>,
    pub jwt_service: Arc<JWTService>,
}

impl UsersFacade {
    pub fn get(&self, user_id: i32) -> TheFuture {
        let future = self.users_service.get(user_id)
            .and_then(|user| {
                serde_json::to_string(&user).map_err(|e| ApiError::from(e))
            })
            .then(|res| match res {
                Ok(data) => future::ok(response_with_json(data)),
                Err(err) => future::ok(response_with_error(err))
            });

        Box::new(future)
    }

    pub fn list(&self, req: TheRequest) -> TheFuture {
        let users_service = self.users_service.clone();

        let future = req.uri().query()
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
            .into_future()
            .and_then(move |(from, count)| {
                users_service.list(from, count)
            })
            .and_then(|user| {
                serde_json::to_string(&user).map_err(|e| ApiError::from(e))
            })
            .then(|res| match res {
                Ok(data) => future::ok(response_with_json(data)),
                Err(err) => future::ok(response_with_error(err))
            });

        Box::new(future)
    }

    pub fn create(&self, req: TheRequest) -> TheFuture {
        let users_service = self.users_service.clone();

        let future = read_body(req).and_then(move |body| {
            serde_json::from_str::<NewUser>(&body)
                .map_err(|e| ApiError::from(e))
                .and_then(|payload| match payload.validate() {
                    Ok(_) => Ok(payload),
                    Err(e) => Err(ApiError::from(e))
                })
                .into_future()
                .and_then(move |payload| {
                    users_service.create(payload)
                })
                .and_then(|user| {
                    serde_json::to_string(&user).map_err(|e| ApiError::from(e))
                })
                .then(|res| match res {
                    Ok(data) => future::ok(response_with_json(data)),
                    Err(err) => future::ok(response_with_error(err))
                })
        });

        Box::new(future)
    }

    pub fn update(&self, req: TheRequest, user_id: i32) -> TheFuture {
        let users_service = self.users_service.clone();

        let future = read_body(req).and_then(move |body| {
            serde_json::from_str::<UpdateUser>(&body)
                .map_err(|e| ApiError::from(e))
                .and_then(|payload| match payload.validate() {
                    Ok(_) => Ok(payload),
                    Err(e) => Err(ApiError::from(e))
                })
                .into_future()
                .and_then(move |payload| {
                    users_service.update(user_id, payload)
                })
                .and_then(|user| {
                    serde_json::to_string(&user).map_err(|e| ApiError::from(e))
                })
                .then(|res| match res {
                    Ok(data) => future::ok(response_with_json(data)),
                    Err(err) => future::ok(response_with_error(err))
                })
        });

        Box::new(future)
    }

    pub fn deactivate(&self, user_id: i32) -> TheFuture {
        let future = self.users_service.deactivate(user_id)
            .and_then(|_user| {
                let message = StatusMessage::new("User has been deactivated");
                serde_json::to_string(&message).map_err(|e| ApiError::from(e))
            })
            .then(|res| match res {
                Ok(data) => future::ok(response_with_json(data)),
                Err(err) => future::ok(response_with_error(err))
            });

        Box::new(future)
    }


     
    pub fn create_token_by_email(&self, req: TheRequest) -> TheFuture {
        let jwt_service = self.jwt_service.clone();

        let future = read_body(req).and_then(move |body| {
            serde_json::from_str::<NewUser>(&body)
                .map_err(|e| ApiError::from(e))
                .and_then(|payload| match payload.validate() {
                    Ok(_) => Ok(payload),
                    Err(e) => Err(ApiError::from(e))
                })
                .into_future()
                .and_then(move |payload| {
                    jwt_service.create_token_email(payload)
                })
                .and_then(|token| {
                    serde_json::to_string(&token).map_err(|e| ApiError::from(e))
                })
                .then(|res| match res {
                    Ok(data) => future::ok(response_with_json(data)),
                    Err(err) => future::ok(response_with_error(err))
                })
        });

        Box::new(future)
    }

    pub fn create_token_by_google(&self, req: TheRequest) -> TheFuture {
        let jwt_service = self.jwt_service.clone();

        let future = read_body(req).and_then(move |body| {
            serde_json::from_str::<ProviderOauth>(&body)
                .map_err(|e| ApiError::from(e))
                .into_future()
                .and_then(move |payload| {
                    jwt_service.create_token_google(payload)
                })
                .and_then(|token| {
                    serde_json::to_string(&token).map_err(|e| ApiError::from(e))
                })
                .then(|res| match res {
                    Ok(data) => future::ok(response_with_json(data)),
                    Err(err) => future::ok(response_with_error(err))
                })
        });

        Box::new(future)
    }

    pub fn create_token_by_facebook(&self, req: TheRequest) -> TheFuture {
        let jwt_service = self.jwt_service.clone();

        let future = read_body(req).and_then(move |body| {
            serde_json::from_str::<ProviderOauth>(&body)
                .map_err(|e| ApiError::from(e))
                .into_future()
                .and_then(move |payload| {
                    jwt_service.create_token_facebook(payload)
                })
                .and_then(|token| {
                    serde_json::to_string(&token).map_err(|e| ApiError::from(e))
                })
                .then(|res| match res {
                    Ok(data) => future::ok(response_with_json(data)),
                    Err(err) => future::ok(response_with_error(err))
                })
        });

        Box::new(future)
    }
}
