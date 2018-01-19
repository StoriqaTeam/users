use hyper;
use serde_json;

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
            ServiceError::Parse(_) => Error::UnprocessableEntity,
            ServiceError::Database(_) => Error::InternalServerError,
            ServiceError::HttpClient(_) => Error::InternalServerError,
            ServiceError::Unknown(_) => Error::InternalServerError
        }
    }
}

impl Error {
    pub fn code(&self) -> hyper::StatusCode {
        use super::error::Error::*;
        use hyper::StatusCode;

        match self {
            &NotFound => StatusCode::NotFound,
            &BadRequest(_) => StatusCode::BadRequest,
            &UnprocessableEntity => StatusCode::UnprocessableEntity,
            &InternalServerError => StatusCode::InternalServerError,
        }
    }

    pub fn message(&self) -> String {
        use super::error::Error::*;

        match self {
            &NotFound => "Not found".to_string(),
            &BadRequest(ref msg) => msg.to_string(),
            &UnprocessableEntity => "Unprocessable Entity".to_string(),
            &InternalServerError => "Internal server Error".to_string(),
        }
    }
}
