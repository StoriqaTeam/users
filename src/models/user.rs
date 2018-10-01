//! Models for working with users

use std::borrow::Cow;
use std::collections::HashMap;
use std::time::SystemTime;

use chrono::NaiveDate;
use regex::Regex;
use validator::{Validate, ValidationError};

use stq_static_resources::Gender;
use stq_types::UserId;

use models::NewIdentity;
use schema::users;

pub fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    lazy_static! {
        static ref PHONE_VALIDATION_RE: Regex = Regex::new(r"^\+?\d{7}\d*$").unwrap();
    }

    if PHONE_VALIDATION_RE.is_match(phone) {
        Ok(())
    } else {
        Err(ValidationError {
            code: Cow::from("phone"),
            message: Some(Cow::from("Incorrect phone format")),
            params: HashMap::new(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Clone, PartialEq)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub email_verified: bool,
    pub phone: Option<String>,
    pub phone_verified: bool,
    pub is_active: bool,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub gender: Option<Gender>,
    pub birthdate: Option<NaiveDate>,
    pub last_login_at: SystemTime,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub saga_id: String,
    pub avatar: Option<String>,
    pub is_blocked: bool,
}

/// Payload for creating users
#[derive(Debug, Serialize, Deserialize, Insertable, Validate, Clone)]
#[table_name = "users"]
pub struct NewUser {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(custom = "validate_phone")]
    pub phone: Option<String>,
    #[validate(length(min = "1", message = "First name must not be empty"))]
    pub first_name: Option<String>,
    #[validate(length(min = "1", message = "Last name must not be empty"))]
    pub last_name: Option<String>,
    #[validate(length(min = "1", message = "Middle name must not be empty"))]
    pub middle_name: Option<String>,
    pub gender: Option<Gender>,
    pub birthdate: Option<NaiveDate>,
    pub last_login_at: SystemTime,
    pub saga_id: String,
}

/// Payload for updating users
#[derive(Default, Debug, Serialize, Deserialize, Insertable, Validate, AsChangeset)]
#[table_name = "users"]
pub struct UpdateUser {
    #[validate(custom = "validate_phone")]
    pub phone: Option<String>,
    #[validate(length(min = "1", message = "First name must not be empty"))]
    pub first_name: Option<String>,
    #[validate(length(min = "1", message = "Last name must not be empty"))]
    pub last_name: Option<String>,
    #[validate(length(min = "1", message = "Middle name must not be empty"))]
    pub middle_name: Option<String>,
    pub gender: Option<Gender>,
    pub birthdate: Option<NaiveDate>,
    pub avatar: Option<String>,
    pub is_active: Option<bool>,
    pub email_verified: Option<bool>,
}

impl UpdateUser {
    pub fn is_empty(&self) -> bool {
        self.phone.is_none()
            && self.first_name.is_none()
            && self.last_name.is_none()
            && self.middle_name.is_none()
            && self.gender.is_none()
            && self.birthdate.is_none()
    }
}

impl From<NewIdentity> for NewUser {
    fn from(identity: NewIdentity) -> Self {
        NewUser {
            email: identity.email,
            phone: None,
            first_name: None,
            last_name: None,
            middle_name: None,
            gender: None,
            birthdate: None,
            last_login_at: SystemTime::now(),
            saga_id: identity.saga_id,
        }
    }
}

/// Payload for searching for user
#[derive(Debug, Serialize, Deserialize)]
pub struct UsersSearchTerms {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_blocked: Option<bool>,
}
