//! Models for password reset
use std::time::SystemTime;

table! {
    reset_tokens (token) {
        token -> VarChar,
        email -> VarChar,
        created_at -> Timestamp,
    }
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug)]
#[table_name = "reset_tokens"]
pub struct ResetToken {
    pub token: String,
    pub email: String,
    pub created_at: SystemTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetRequest {
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetApply {
    pub token: String,
    pub password: String,
}
