use serde_json;
use diesel;
use hyper::StatusCode;

/*
pub enum Error {
    Default(JsonError),
    Json(serde_json::error::Error),
    Database(diesel::result::Error)
}

impl Error {
    pub fn new(message: &str) -> Error {
        let json_error = JsonError { error: message.into() };
        Error::Default(json_error)
    }
}
*/

// Error
#[derive(Debug)]
pub enum Error {
    NotFound,
    BadRequest,
    UnprocessableEntity,
    InternalServerError,
}

impl Error {
    pub fn to_code(&self) -> StatusCode {
        use error::Error::*;

        match self {
            &NotFound => StatusCode::NotFound,
            &BadRequest => StatusCode::BadRequest,
            &UnprocessableEntity => StatusCode::UnprocessableEntity,
            &InternalServerError => StatusCode::InternalServerError,
        }
    }

    pub fn to_string(&self) -> String {
        use error::Error::*;

        match self {
            &NotFound => format!("Entity not found"),
            &BadRequest => format!("Bad request"),
            &UnprocessableEntity => format!("Failure during JSON conversion"),
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
