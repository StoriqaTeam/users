use r2d2::Error as R2D2Error;
use diesel::result::Error as DieselError;

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
    Connection(#[cause] R2D2Error),
    #[fail(display = "Diesel transaction error")]
    Transaction(#[cause] DieselError),
    #[fail(display = "Repo error")]
    Database(#[cause] RepoError),
    #[fail(display = "Http client error: {}", _0)]
    HttpClient(String),
    #[fail(display = "Email already exists: [{}]", _0)]
    EmailAlreadyExists(String),
    #[fail(display = "Incorrect email or password")]
    IncorrectCredentials,
    #[fail(display = "Unauthorized")]
    Unauthorized(String),
    #[fail(display = "Unknown error: {}", _0)]
    Unknown(String),
}

impl From<RepoError> for ServiceError {
    fn from(err: RepoError) -> Self {
        match err {
            RepoError::NotFound => ServiceError::NotFound,
            RepoError::Rollback => ServiceError::Rollback,
            RepoError::ContstaintViolation(msg, e) => ServiceError::Database(RepoError::ContstaintViolation(msg, e)),
            RepoError::MismatchedType(msg, e) => ServiceError::Database(RepoError::MismatchedType(msg, e)),
            RepoError::Connection(msg, e) => ServiceError::Database(RepoError::Connection(msg, e)),
            RepoError::Unknown(msg, e) => ServiceError::Database(RepoError::Unknown(msg, e)),
            RepoError::Unauthorized(res, act) => ServiceError::Unauthorized(format!(
                "Unauthorized access: Resource: {}, Action: {}",
                res, act
            )),
        }
    }
}

impl From<DieselError> for ServiceError {
    fn from(err: DieselError) -> Self {
        ServiceError::Transaction(err)
    }
}
