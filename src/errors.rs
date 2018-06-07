use hyper::StatusCode;
use serde_json;
use validator::ValidationErrors;

use stq_http::errors::{Codeable, PayloadCarrier};

#[derive(Debug, Fail)]
pub enum ControllerError {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Parse error")]
    Parse,
    #[fail(display = "Validation error")]
    Validate(ValidationErrors),
    #[fail(display = "Server is refusing to fullfil the reqeust")]
    Forbidden,
    #[fail(display = "R2D2 connection error")]
    Connection,
    #[fail(display = "Http Client error")]
    HttpClient,
    #[fail(display = "Invalid oauth token")]
    InvalidToken,
}

impl Codeable for ControllerError {
    fn code(&self) -> StatusCode {
        match *self {
            ControllerError::NotFound => StatusCode::NotFound,
            ControllerError::Validate(_) => StatusCode::BadRequest,
            ControllerError::Parse => StatusCode::UnprocessableEntity,
            ControllerError::Connection | ControllerError::HttpClient => StatusCode::InternalServerError,
            ControllerError::Forbidden | ControllerError::InvalidToken => StatusCode::Forbidden,
        }
    }
}

impl PayloadCarrier for ControllerError {
    fn payload(&self) -> Option<serde_json::Value> {
        match *self {
            ControllerError::Validate(ref e) => serde_json::to_value(e.clone()).ok(),
            _ => None,
        }
    }
}
