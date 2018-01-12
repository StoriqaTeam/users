use validator::Validate;

/// Payload for creating JWT token by provider
#[derive(Serialize, Deserialize, Validate)]
pub struct ProviderOauth {
    pub token: String,
}