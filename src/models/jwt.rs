/// Json Web Token Model
#[derive(Debug, Serialize, Queryable, Deserialize)]
pub struct JWT {
    pub token: String,
}