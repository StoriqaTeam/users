use hyper;
use serde_json;
use diesel;
use validator::ValidationErrors;

use responses::error::ErrorMessage;
use services::error::Error as ServiceError;

#[derive(Debug)]
pub enum Error {
    NotFound,
    BadRequest(String),
    UnprocessableEntity,
    InternalServerError,
}

impl From<serde_json::error::Error> for Error {
    fn from(_e: serde_json::error::Error) -> Self {
        Error::UnprocessableEntity
    }
}

impl From<ServiceError> for Error {
    fn from(e: ServiceError) -> Self {
        match e {
            ServiceError::NotFound => Error::NotFound,
            ServiceError::Rollback => Error::BadRequest("Transaction rollback".to_string()),
            ServiceError::Validate(msg) => Error::BadRequest(format!("{}", msg)),
            ServiceError::Parse(msg) => Error::UnprocessableEntity,
            ServiceError::Database(_) => Error::InternalServerError,
            ServiceError::HttpClient(_) => Error::InternalServerError,
            ServiceError::Unknown(_) => Error::InternalServerError
        }
    }
}
