use diesel::result::Error as DieselError;
use stq_acl;
use models::authorization::*;

use failure::Error;

#[derive(Debug, Fail)]
pub enum RepoError {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Rollback")]
    Rollback,
    #[fail(display = "Unauthorized")]
    Unauthorized(Resource, Action),
    #[fail(display = "Constraint violation")]
    ContstaintViolation(Error),
    #[fail(display = "Mismatched type")]
    MismatchedType(Error),
    #[fail(display = "Connection")]
    Connection(Error),
    #[fail(display = "Unknown")]
    Unknown(Error),
}

impl From<DieselError> for RepoError {
    fn from(err: DieselError) -> Self {
        match err {
            DieselError::InvalidCString(e) => RepoError::Unknown(DieselError::InvalidCString(e).into()),
            DieselError::DatabaseError(kind, info) => RepoError::ContstaintViolation(DieselError::DatabaseError(kind, info).into()),
            DieselError::NotFound => RepoError::NotFound,
            DieselError::QueryBuilderError(e) => RepoError::Unknown(DieselError::QueryBuilderError(e).into()),
            DieselError::SerializationError(e) => RepoError::MismatchedType(DieselError::SerializationError(e).into()),
            DieselError::DeserializationError(e) => RepoError::MismatchedType(DieselError::DeserializationError(e).into()),
            DieselError::RollbackTransaction => RepoError::Rollback,
            _ => RepoError::Unknown(DieselError::__Nonexhaustive.into()),
        }
    }
}
