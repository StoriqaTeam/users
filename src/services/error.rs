use diesel::result::Error as DieselError;

use failure::Error;

use stq_http::errors::ControllerError;

use validator::ValidationErrors;
use http::client::Error as HttpError;
use repos::error::RepoError;

#[derive(Debug, Fail)]
pub enum ServiceError {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Rollback")]
    Rollback,
    #[fail(display = "Validation error: {}", _0)]
    Validate(ValidationErrors),
    #[fail(display = "Parse error: {}", _0)]
    Parse(String),
    #[fail(display = "R2D2 connection error")]
    Connection(Error),
    #[fail(display = "Diesel transaction error")]
    Transaction(Error),
    #[fail(display = "Repo error")]
    Database(Error),
    #[fail(display = "Http client error: {}", _0)]
    HttpClient(String),
    #[fail(display = "Email already exists: [{}]", _0)]
    EmailAlreadyExists(String),
    #[fail(display = "Incorrect email or password")]
    IncorrectCredentials,
    #[fail(display = "Unauthorized")]
    Unauthorized(String),
    #[fail(display = "Invalid token")]
    InvalidToken,
    #[fail(display = "Unknown error: {}", _0)]
    Unknown(String),
}

impl From<RepoError> for ServiceError {
    fn from(err: RepoError) -> Self {
        match err {
            RepoError::NotFound => ServiceError::NotFound,
            RepoError::Rollback => ServiceError::Rollback,
            RepoError::ContstaintViolation(e) => ServiceError::Database(RepoError::ContstaintViolation(e).into()),
            RepoError::MismatchedType(e) => ServiceError::Database(RepoError::MismatchedType(e).into()),
            RepoError::Connection(e) => ServiceError::Database(RepoError::Connection(e).into()),
            RepoError::InvalidToken => ServiceError::InvalidToken,
            RepoError::Unknown(e) => ServiceError::Database(RepoError::Unknown(e).into()),
            RepoError::Unauthorized(res, act) => ServiceError::Unauthorized(format!(
                "Unauthorized access: Resource: {}, Action: {}",
                res, act
            )),
        }
    }
}

impl From<DieselError> for ServiceError {
    fn from(err: DieselError) -> Self {
        ServiceError::Transaction(err.into())
    }
}

impl From<HttpError> for ServiceError {
    fn from(err: HttpError) -> Self {
        ServiceError::HttpClient(format!("{:?}", err))
    }
}

impl From<ServiceError> for ControllerError {
    fn from(e: ServiceError) -> Self {
        match e {
            ServiceError::NotFound => ControllerError::NotFound,
            ServiceError::Rollback => ControllerError::BadRequest(ServiceError::Rollback.into()),
            ServiceError::Validate(valid_err) => ControllerError::Validate(valid_err),
            ServiceError::Unauthorized(msg) => ControllerError::BadRequest(ServiceError::Unauthorized(msg).into()),
            ServiceError::Parse(msg) => ControllerError::UnprocessableEntity(ServiceError::Parse(msg).into()),
            ServiceError::Database(msg) => ControllerError::InternalServerError(ServiceError::Database(msg).into()),
            ServiceError::HttpClient(msg) => ControllerError::InternalServerError(ServiceError::HttpClient(msg).into()),
            ServiceError::EmailAlreadyExists(msg) => ControllerError::BadRequest(ServiceError::EmailAlreadyExists(msg).into()),
            ServiceError::InvalidToken => ControllerError::BadRequest(ServiceError::InvalidToken.into()),
            ServiceError::IncorrectCredentials => ControllerError::BadRequest(ServiceError::IncorrectCredentials.into()),
            ServiceError::Connection(msg) => ControllerError::InternalServerError(ServiceError::Connection(msg).into()),
            ServiceError::Transaction(msg) => ControllerError::InternalServerError(ServiceError::Transaction(msg).into()),
            ServiceError::Unknown(msg) => ControllerError::InternalServerError(ServiceError::Unknown(msg).into()),
        }
    }
}
