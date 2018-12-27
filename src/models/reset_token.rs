//! Models for password reset
use std::fmt;
use std::time::SystemTime;

use base64::encode;
use uuid::Uuid;
use validator::Validate;

use stq_static_resources::TokenType;

use models::user::User;
use schema::reset_tokens;

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug)]
#[table_name = "reset_tokens"]
pub struct ResetToken {
    pub token: String,
    pub email: String,
    pub created_at: SystemTime,
    pub token_type: TokenType,
    pub uuid: Uuid,
    pub updated_at: SystemTime,
}

impl ResetToken {
    pub fn new(email: String, token_type: TokenType, uuid: Option<Uuid>) -> ResetToken {
        let uuid = uuid.unwrap_or(Uuid::new_v4());
        let token = encode(&Uuid::new_v4().to_string());
        ResetToken {
            token,
            email,
            token_type,
            uuid,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
}

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct ResetRequest {
    #[validate(email(code = "not_valid", message = "Invalid email format"))]
    pub email: String,
    pub uuid: Uuid,
}

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct VerifyRequest {
    #[validate(email(code = "not_valid", message = "Invalid email format"))]
    pub email: String,
}

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct ResetApply {
    pub token: String,
    #[validate(length(min = "8", max = "30", message = "Password should be between 8 and 30 symbols"))]
    pub password: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ResetApplyToken {
    pub email: String,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResetMail {
    pub to: String,
    pub subject: String,
    pub text: String,
}

impl fmt::Display for ResetApply {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ResetApply {{ token: \"{}\", password: \"*****\" }}", self.token)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailVerifyApplyToken {
    pub user: User,
    pub token: String,
}
