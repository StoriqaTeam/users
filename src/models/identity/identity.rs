//! Module containing info about Identity models
use validator::Validate;

use models::{Provider, UserId};

use uuid::Uuid;

table! {
    use diesel::sql_types::*;
    identities (user_id) {
        user_id -> Integer,
        email -> Varchar,
        password -> Nullable<VarChar>,
        provider -> Varchar,
        saga_id -> VarChar,
    }
}

/// Payload for creating identity for users
#[derive(Debug, Serialize, Deserialize, Validate, Insertable, Queryable, Clone)]
#[table_name = "identities"]
pub struct Identity {
    pub user_id: UserId,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = "8", max = "30", message = "Password should be between 8 and 30 symbols"))]
    pub password: Option<String>,
    pub provider: Provider,
    pub saga_id: String,
}

/// Payload for creating users
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct NewIdentity {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = "8", max = "30", message = "Password should be between 8 and 30 symbols"))]
    pub password: Option<String>,
    pub provider: Provider,
    pub saga_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct NewEmailIdentity {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = "8", max = "30", message = "Password should be between 8 and 30 symbols"))]
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct ChangeIdentityPassword {
    pub old_password: String,
    #[validate(length(min = "8", max = "30", message = "Password should be between 8 and 30 symbols"))]
    pub new_password: String,
}

/// Payload for updating identity password
#[derive(Clone, Debug, Serialize, Deserialize, Insertable, Validate, AsChangeset)]
#[table_name = "identities"]
pub struct UpdateIdentity {
    #[validate(length(min = "8", max = "30", message = "Password should be between 8 and 30 symbols"))]
    pub password: Option<String>,
}

impl From<NewEmailIdentity> for NewIdentity {
    fn from(v: NewEmailIdentity) -> Self {
        Self {
            email: v.email,
            password: Some(v.password),
            provider: Provider::UnverifiedEmail,
            saga_id: Uuid::new_v4().to_string(),
        }
    }
}
