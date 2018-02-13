use hyper;
use serde_json;

use failure::Fail;
use failure::Error;

use services::error::ServiceError;

#[derive(Debug, Fail)]
pub enum ControllerError {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Parse error")]
    Parse(String),
    #[fail(display = "Bad request")]
    BadRequest(Error),
    #[fail(display = "Unprocessable entity")]
    UnprocessableEntity(Error),
    #[fail(display = "Internal server error")]
    InternalServerError(Error),
}

impl From<serde_json::error::Error> for ControllerError {
    fn from(e: serde_json::error::Error) -> Self {
        ControllerError::UnprocessableEntity(e.into())
    }
}

impl From<ServiceError> for ControllerError {
    fn from(e: ServiceError) -> Self {
        match e {
            ServiceError::NotFound => ControllerError::NotFound,
            ServiceError::Rollback => ControllerError::BadRequest(ServiceError::Rollback.into()),
            ServiceError::Validate(msg) => ControllerError::BadRequest(ServiceError::Validate(msg).into()),
            ServiceError::Parse(msg) => ControllerError::UnprocessableEntity(ServiceError::Parse(msg).into()),
            ServiceError::Database(msg) => ControllerError::InternalServerError(ServiceError::Database(msg).into()),
            ServiceError::HttpClient(msg) => ControllerError::InternalServerError(ServiceError::HttpClient(msg).into()),
            ServiceError::EmailAlreadyExists(msg) => ControllerError::BadRequest(ServiceError::EmailAlreadyExists(msg).into()),
            ServiceError::IncorrectCredentials => ControllerError::BadRequest(ServiceError::IncorrectCredentials.into()),
            ServiceError::Connection(msg) => ControllerError::InternalServerError(ServiceError::Connection(msg).into()),
            ServiceError::Transaction(msg) => ControllerError::InternalServerError(ServiceError::Transaction(msg).into()),
            ServiceError::Unknown(msg) => ControllerError::InternalServerError(ServiceError::Unknown(msg).into()),
        }
    }
}

impl ControllerError {
    /// Converts `Error` to HTTP Status Code
    pub fn code(&self) -> hyper::StatusCode {
        use super::error::ControllerError::*;
        use hyper::StatusCode;

        match self {
            &NotFound => StatusCode::NotFound,
            &Parse(_) => StatusCode::BadRequest,
            &BadRequest(_) => StatusCode::BadRequest,
            &UnprocessableEntity(_) => StatusCode::UnprocessableEntity,
            &InternalServerError(_) => StatusCode::InternalServerError,
        }
    }

    /// Converts `Error` to string
    pub fn message(&self) -> String {
        use super::error::ControllerError::*;

        match self {
            &NotFound => "Not found".to_string(),
            &Parse(_) => "Bad request".to_string(),
            &BadRequest(_) => "Bad request".to_string(),
            &UnprocessableEntity(_) => "Unprocessable entity".to_string(),
            &InternalServerError(_) => "Internal server error".to_string(),
        }
    }
}
