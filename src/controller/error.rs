use hyper;
use serde_json;

use services::error::ServiceError;

#[derive(Debug, Fail)]
pub enum ControllerError {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Bad request: {}", _0)]
    BadRequest(String),
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
            ServiceError::Rollback => ControllerError::BadRequest("Transaction rollback".to_string()),
            ServiceError::Validate(msg) => ControllerError::BadRequest(
                serde_json::to_string(&msg).unwrap_or("Unable to serialize validation errors".to_string())
            ),
            ServiceError::Parse(msg) => ControllerError::UnprocessableEntity(format!("Parse error: {}", msg)),
            ServiceError::Database(msg) => ControllerError::InternalServerError(ServiceError::Database(msg)),
            ServiceError::HttpClient(msg) => ControllerError::InternalServerError(ServiceError::HttpClient(msg)),
            ServiceError::EmailAlreadyExistsError(msg) => ControllerError::BadRequest(msg),
            ServiceError::IncorrectCredentialsError => ControllerError::BadRequest(
                "Incorrect email or password".to_string()),
            _ => ControllerError::BadRequest("Unknown".into())
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
            &BadRequest(_) => "Bad request".to_string(),
            &UnprocessableEntity(_) => "Unprocessable entity".to_string(),
            &InternalServerError(_) => "Internal server error".to_string(),
        }
    }
}
