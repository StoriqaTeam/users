//! Models for password reset
pub mod token_type;

pub use self::token_type::TokenType;

use std::time::SystemTime;

use validator::Validate;

table! {
    reset_tokens (token) {
        token -> VarChar,
        email -> VarChar,
        token_type -> VarChar,
        created_at -> Timestamp,
    }
}

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug)]
#[table_name = "reset_tokens"]
pub struct ResetToken {
    pub token: String,
    pub email: String,
    pub token_type: TokenType,
    pub created_at: SystemTime,
}

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct ResetRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct ResetApply {
    pub token: String,
    #[validate(length(min = "8", max = "30", message = "Password should be between 8 and 30 symbols"))]
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResetMail {
    pub to: String,
    pub subject: String,
    pub text: String,
}
