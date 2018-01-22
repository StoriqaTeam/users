use hyper;
use serde_json;

use services::error::Error as ServiceError;

#[derive(Debug)]
pub enum Error {
    NotFound,
    BadRequest(String),
    UnprocessableEntity(String),
    InternalServerError,
}

impl From<serde_json::error::Error> for Error {
    fn from(e: serde_json::error::Error) -> Self {
        Error::UnprocessableEntity(format!("{}", e).to_string())
    }
}

impl From<ServiceError> for Error {
    fn from(e: ServiceError) -> Self {
        match e {
            ServiceError::NotFound => Error::NotFound,
            ServiceError::Rollback => Error::BadRequest("Transaction rollback".to_string()),
            ServiceError::Validate(msg) => Error::BadRequest(serde_json::to_string(&msg).unwrap_or("Unable to serialize validation errors".to_string())),
            ServiceError::Parse(e) => Error::UnprocessableEntity(format!("Parse error: {}", e)),
            ServiceError::Database(_) => Error::InternalServerError,
            ServiceError::HttpClient(_) => Error::InternalServerError,
            ServiceError::Unknown(_) => Error::InternalServerError
        }
    }
}

impl Error {
    /// Converts `Error` to HTTP Status Code
    ///
    /// ```
    /// # extern crate hyper;
    /// # extern crate users_lib;
    /// 
    /// # use users_lib::error::Error;
    /// # use hyper::StatusCode;
    /// 
    /// # fn main() {
    ///     let mut error = Error::NotFound.to_code();
    ///     assert_eq!(error, StatusCode::NotFound);
    /// 
    ///     error = Error::BadRequest("bad".to_string()).to_code();
    ///     assert_eq!(error, StatusCode::BadRequest);
    /// 
    ///     error = Error::UnprocessableEntity.to_code();
    ///     assert_eq!(error, StatusCode::UnprocessableEntity);
    /// 
    ///     error = Error::InternalServerError.to_code();
    ///     assert_eq!(error, StatusCode::InternalServerError);
    /// # }
    /// ```
    pub fn code(&self) -> hyper::StatusCode {
        use super::error::Error::*;
        use hyper::StatusCode;

        match self {
            &NotFound => StatusCode::NotFound,
            &BadRequest(_) => StatusCode::BadRequest,
            &UnprocessableEntity(_) => StatusCode::UnprocessableEntity,
            &InternalServerError => StatusCode::InternalServerError,
        }
    }


    /// Converts `Error` to string
    /// 
    /// ```
    /// use users_lib::error::Error;
    /// 
    /// let mut error = Error::NotFound.message();
    /// assert_eq!(error, "Entity not found".to_string());
    /// 
    /// error = Error::BadRequest("bad".to_string()).message();
    /// assert_eq!(error, "bad".to_string());
    /// 
    /// error = Error::UnprocessableEntity.message();
    /// assert_eq!(error, "Serialization error".to_string());
    /// 
    /// error = Error::InternalServerError.message();
    /// assert_eq!(error, "Internal server error".to_string());
    /// ```
    pub fn message(&self) -> String {
        use super::error::Error::*;

        match self {
            &NotFound => "Not found".to_string(),
            &BadRequest(ref msg) => msg.to_string(),
            &UnprocessableEntity(ref msg) => msg.to_string(),
            &InternalServerError => "Internal server Error".to_string(),
        }
    }
}
