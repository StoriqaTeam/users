//! Models for managing Json Web Token

/// Json Web Token Model sent back to gateway
#[derive(Debug, Serialize, Queryable, Deserialize)]
pub struct JWT {
    pub token: String,
}

/// Payload received from gateway for creating JWT token by provider
#[derive(Serialize, Deserialize)]
pub struct ProviderOauth {
    pub token: String,
}


/// Json web token payload 
#[derive(Serialize, Deserialize, Debug)]
pub struct JWTPayload {
    pub user_id: i32,
}

impl JWTPayload {
    pub fn new(id: i32) -> Self {
        Self {
            user_id: id,
        }
    }
}
