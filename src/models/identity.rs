//! Models for working with identities
use std::fmt;

use uuid::Uuid;
use validator::Validate;

use stq_static_resources::Provider;
use stq_types::UserId;

use schema::identities;

/// Payload for creating identity for users
#[derive(Debug, Serialize, Deserialize, Validate, Queryable, Insertable, Clone)]
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
pub struct EmailIdentity {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
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
    pub provider: Option<Provider>,
}

impl From<EmailIdentity> for NewIdentity {
    fn from(v: EmailIdentity) -> Self {
        Self {
            email: v.email,
            password: Some(v.password),
            provider: Provider::Email,
            saga_id: Uuid::new_v4().to_string(),
        }
    }
}

impl fmt::Display for EmailIdentity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EmailIdentity {{ email: \"{}\", password: \"******\" }}", self.email)
    }
}
