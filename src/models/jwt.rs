/// Json Web Token Model
#[derive(Debug, Serialize, Queryable, Deserialize)]
pub struct JWT {
    pub token: String,
}

/// Payload for creating JWT token by provider
#[derive(Serialize, Deserialize)]
pub struct ProviderOauth {
    pub code: String,
}