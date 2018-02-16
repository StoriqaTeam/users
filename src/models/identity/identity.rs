//! Module containing info about Identity models
use validator::Validate;

use models::{Provider, UserId};

table! {
    use diesel::sql_types::*;
    identities (user_id) {
        user_id -> Integer,
        email -> Varchar,
        password -> Nullable<Varchar>,
        provider -> Nullable<Varchar>,
    }
}

/// Payload for creating identity for users
#[derive(Debug, Serialize, Deserialize, Validate, Insertable, Queryable)]
#[table_name = "identities"]
pub struct Identity {
    pub user_id: UserId,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = "8", max = "30", message = "Password should be between 8 and 30 symbols"))]
    pub password: Option<String>,
    pub provider: Provider,
}

/// Payload for creating users
#[derive(Serialize, Deserialize, Validate, Clone)]
pub struct NewIdentity {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = "8", max = "30", message = "Password should be between 8 and 30 symbols"))]
    pub password: String,
}
