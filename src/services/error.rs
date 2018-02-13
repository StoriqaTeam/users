use diesel::result::Error as DieselError;

use failure::Error;

use validator::ValidationErrors;
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
            RepoError::Unknown(e) => ServiceError::Database(RepoError::Unknown(e).into()),
        }
    }
}

impl From<DieselError> for ServiceError {
    fn from(err: DieselError) -> Self {
        ServiceError::Transaction(err.into())
    }
}
