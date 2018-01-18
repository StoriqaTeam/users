use hyper;
use super::controller::error::Error as ControllerError;

struct Error(hyper::Error);

impl Error {
    /// Converts `Error` to HTTP Status Code
    pub fn to_code(&self) -> hyper::StatusCode {
        use self::ControllerError::*;
        use hyper::StatusCode;

        match self.0 {
            &NotFound => StatusCode::NotFound,
            &BadRequest(_) => StatusCode::BadRequest,
            &UnprocessableEntity => StatusCode::UnprocessableEntity,
            &InternalServerError => StatusCode::InternalServerError,
        }
    }

    /// Converts `Error` to string
    pub fn to_string(&self) -> String {
        use self::ControllerError::*;

        match self.0 {
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
