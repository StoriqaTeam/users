/// Payload for creating JWT token by provider
#[derive(Serialize, Deserialize, Insertable, Validate)]
pub struct ProviderOauth {
    pub token: String,
}