use diesel::result::Error as DieselError;

/// Repos layer Error
pub enum Error {
  NotFound,
  Rollback,
  ContstaintViolation(String),
  MismatchedType(String),
  Connection(String),
  Unknown(String)
}

impl From<DieselError> for Error {
    fn from(err: DieselError) -> Self {
        match err {
            DieselError::InvalidCString(e) => Error::Unknown(format!("{}", e)),
            DieselError::DatabaseError(kind, info) => Error::ContstaintViolation(format!("{:?}: {:?}", kind, info)),
            DieselError::NotFound => Error::NotFound,
            DieselError::QueryBuilderError(e) => Error::Unknown(format!("{}", e)),
            DieselError::SerializationError(e) => Error::MismatchedType(format!("{}", e)),
            DieselError::DeserializationError(e) => Error::MismatchedType(format!("{}", e)),
            DieselError::RollbackTransaction => Error::Rollback,
            _ => Error::Unknown("Unknown diesel error".to_string())
        }
    }
}
