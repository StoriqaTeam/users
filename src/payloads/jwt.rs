/// Payload for creating JWT token by provider
#[derive(Serialize, Deserialize)]
pub struct ProviderOauth {
    pub token: String,
}
