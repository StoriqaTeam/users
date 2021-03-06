//! Models for managing Json Web Token

use stq_static_resources::Provider;
use stq_types::{Alpha3, UserId};

/// Json Web Token created by provider user status
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum UserStatus {
    New(UserId),
    Exists,
}

/// Json Web Token Model sent back to gateway created by email and password
#[derive(Clone, Debug, Serialize, Deserialize, Queryable)]
pub struct JWT {
    pub token: String,
    pub status: UserStatus,
}

/// Payload received from gateway for creating JWT token by provider
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderOauth {
    pub token: String,
    pub additional_data: Option<NewUserAdditionalData>,
}

/// Json web token payload
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JWTPayload {
    pub user_id: UserId,
    pub exp: i64,
    pub provider: Provider,
}

impl JWTPayload {
    pub fn new(id: UserId, exp_arg: i64, provider_arg: Provider) -> Self {
        Self {
            user_id: id,
            exp: exp_arg,
            provider: provider_arg,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NewUserAdditionalData {
    pub referal: Option<UserId>,
    pub utm_marks: Option<serde_json::Value>,
    pub country: Option<Alpha3>,
    pub referer: Option<String>,
}
