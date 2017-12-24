use hyper::StatusCode;
use serde_json;
use diesel;
use validator::ValidationErrors;

// Error
#[derive(Debug)]
pub enum Error {
    NotFound,
    BadRequest(String),
    UnprocessableEntity,
    InternalServerError,
}

impl Error {
    pub fn to_code(&self) -> StatusCode {
        use error::Error::*;

        match self {
            &NotFound => StatusCode::NotFound,
            &BadRequest(_) => StatusCode::BadRequest,
            &UnprocessableEntity => StatusCode::UnprocessableEntity,
            &InternalServerError => StatusCode::InternalServerError,
        }
    }

    pub fn to_string(&self) -> String {
        use error::Error::*;

        match self {
            &NotFound => format!("Entity not found"),
            &BadRequest(ref messages) => format!("{:?}", messages),
            &UnprocessableEntity => format!("Serialization error"),
            &InternalServerError => format!("Internal server error"),
        }
    }

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

impl From<serde_json::error::Error> for Error {
    fn from(_e: serde_json::error::Error) -> Self {
        Error::UnprocessableEntity
    }
}

impl From<ValidationErrors> for Error {
    fn from(_e: ValidationErrors) -> Self {
        // TODO: Unwrap messages from Vec<Vec<Option>>
        Error::BadRequest("Validation error".to_string())
    }
}

// Error Message
#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorMessage {
    code: u16,
    message: String
}

impl ErrorMessage {
    pub fn new(error: &Error) -> ErrorMessage {
        ErrorMessage {
            code: error.to_code().as_u16(),
            message: error.to_string()
        }
    }
}

// Status Message
#[derive(Serialize, Deserialize, Debug)]
pub struct StatusMessage {
    pub status: String
}

impl StatusMessage {
    pub fn new(msg: &str) -> StatusMessage {
        StatusMessage {
            status: msg.to_string()
        }
    }
}
