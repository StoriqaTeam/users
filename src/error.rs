use hyper;
use serde_json;
use diesel;
use validator::ValidationErrors;

use responses::error::ErrorMessage;
use services::error::Error as ServiceError;

/// Error wrapper for `hyper`, `diesel`, `serde`, `validator`
#[derive(Debug)]
pub enum Error {
    NotFound,
    BadRequest(String),
    UnprocessableEntity,
    InternalServerError,
}

impl Error {
    /// Converts `Error` to HTTP Status Code
    pub fn to_code(&self) -> hyper::StatusCode {
        use error::Error::*;
        use hyper::StatusCode;

        match self {
            &NotFound => StatusCode::NotFound,
            &BadRequest(_) => StatusCode::BadRequest,
            &UnprocessableEntity => StatusCode::UnprocessableEntity,
            &InternalServerError => StatusCode::InternalServerError,
        }
    }

    /// Converts `Error` to string
    pub fn to_string(&self) -> String {
        use error::Error::*;

        match self {
            &NotFound => format!("Entity not found"),
            &BadRequest(ref message) => format!("{}", message),
            &UnprocessableEntity => format!("Serialization error"),
            &InternalServerError => format!("Internal server error"),
        }
    }

    /// Converts `Error` to JSON response body
    pub fn to_json(&self) -> String {
        let message = ErrorMessage::new(self);
        serde_json::to_string(&message).unwrap()
    }
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => Error::NotFound,
            _ => Error::InternalServerError,
        }
    }
}

impl From<hyper::Error> for Error {
    fn from(_e: hyper::Error) -> Self {
        Error::InternalServerError
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(_e: serde_json::error::Error) -> Self {
        Error::UnprocessableEntity
    }
}

impl From<ValidationErrors> for Error {
    fn from(e: ValidationErrors) -> Self {
        // Grabs first validation error
        // TODO: Grab all of them to Vec<String>?

        let message = e.inner().values().next()
            .ok_or("Unreachable validation error")
            .and_then(|vec| vec.first().ok_or("Unreachable validation error"))
            .and_then(|x| x.message.clone().ok_or("Unknown validation error"))
            .and_then(|x| Ok(x.into_owned()));

        match message {
            Ok(msg) => Error::BadRequest(msg),
            Err(err) => Error::BadRequest(err.to_string())
        }
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
