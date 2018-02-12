use diesel::result::Error as DieselError;

#[derive(Debug, Fail)]
pub enum RepoError {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Rollback")]
    Rollback,
    #[fail(display = "Constraint violation: {}", _0)]
    ContstaintViolation(String, #[cause] DieselError),
    #[fail(display = "Mismatched type: {}", _0)]
    MismatchedType(String, #[cause] DieselError),
    #[fail(display = "Connection: {}", _0)]
    Connection(String, #[cause] DieselError),
    #[fail(display = "Unknown: {}", _0)]
    Unknown(String, #[cause] DieselError),
}

impl From<DieselError> for RepoError {
    fn from(err: DieselError) -> Self {
        match err {
            DieselError::InvalidCString(e) => RepoError::Unknown("".to_string(), DieselError::InvalidCString(e)),
            DieselError::DatabaseError(kind, info) => RepoError::ContstaintViolation(
                format!("{:?}: {:?}", kind, info), DieselError::DatabaseError(kind, info)),
            DieselError::NotFound => RepoError::NotFound,
            DieselError::QueryBuilderError(e) => RepoError::Unknown(
                "Query builder error".to_string(), DieselError::QueryBuilderError(e)),
            DieselError::SerializationError(e) => RepoError::MismatchedType(
                "Serialization error".to_string(), DieselError::SerializationError(e)),
            DieselError::DeserializationError(e) => RepoError::MismatchedType(
                "Deserialization error".to_string(), DieselError::DeserializationError(e)),
            DieselError::RollbackTransaction => RepoError::Rollback,
            _ => RepoError::Unknown("Unknown diesel error".to_string(), DieselError::__Nonexhaustive)
        }
    }
}
