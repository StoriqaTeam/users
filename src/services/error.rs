use validator::ValidationErrors;

use ::repos::error::Error as RepoError;

pub enum Error {
    NotFound,
    Rollback,
    Validate(ValidationErrors),
    Parse(String),
    Database(String),
    HttpClient(String),
    Unknown(String)
}

impl From<RepoError> for Error {
    fn from(err: RepoError) -> Self {
        match err {
            RepoError::NotFound => Error::NotFound,
            RepoError::Rollback => Error::Rollback,
            RepoError::ContstaintViolation(msg) => Error::Database(format!("Constraint violation: {}", msg)),
            RepoError::MismatchedType(msg) => Error::Database(format!("Mismatched type: {}", msg)),
            RepoError::Connection(msg) => Error::Database(format!("Connection error: {}", msg)),
            RepoError::Unknown(msg) => Error::Database(format!("Unknown: {}", msg)),
        }
    }
}
