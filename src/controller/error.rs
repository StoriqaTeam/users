use hyper;
use serde_json;

use services::error::ServiceError;

#[derive(Debug, Fail)]
pub enum ControllerError {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Parse error: {}", _0)]
    Parse(String),
    #[fail(display = "Bad request")]
    BadRequest(#[cause] ServiceError),
    #[fail(display = "Unprocessable entity: {}", _0)]
    UnprocessableEntity(String),
    #[fail(display = "Internal server error")]
    InternalServerError(#[cause] ServiceError),
}

impl From<serde_json::error::Error> for ControllerError {
    fn from(e: serde_json::error::Error) -> Self {
        ControllerError::UnprocessableEntity("Serialization error".to_string())
    }
}

impl From<ServiceError> for ControllerError {
    fn from(e: ServiceError) -> Self {
        match e {
            ServiceError::NotFound => ControllerError::NotFound,
            ServiceError::Rollback => ControllerError::BadRequest(ServiceError::Rollback),
            ServiceError::Validate(msg) => ControllerError::BadRequest(ServiceError::Validate(msg)),
            ServiceError::Parse(msg) => ControllerError::UnprocessableEntity(format!("Parse error: {}", msg)),
            ServiceError::Database(msg) => ControllerError::InternalServerError(ServiceError::Database(msg)),
            ServiceError::HttpClient(msg) => ControllerError::InternalServerError(ServiceError::HttpClient(msg)),
            ServiceError::EmailAlreadyExists(msg) => ControllerError::BadRequest(ServiceError::EmailAlreadyExists(msg)),
            ServiceError::IncorrectCredentials => ControllerError::BadRequest(ServiceError::IncorrectCredentials),
            ServiceError::Connection(msg) => ControllerError::InternalServerError(ServiceError::Connection(msg)),
            ServiceError::Transaction(msg) => ControllerError::InternalServerError(ServiceError::Transaction(msg)),
            ServiceError::Unknown(msg) => ControllerError::InternalServerError(ServiceError::Unknown(msg)),
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
