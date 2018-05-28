//! Module containing info about User models
use chrono::NaiveDate;
use std::time::SystemTime;

use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;
use validator::{Validate, ValidationError};

use models::{Gender, NewIdentity, UserId};

pub fn validate_phone(phone: &String) -> Result<(), ValidationError> {
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

table! {
    use diesel::sql_types::*;
    users (id) {
        id -> Integer,
        email -> Varchar,
        email_verified -> Bool,
        phone -> Nullable<VarChar>,
        phone_verified -> Bool,
        is_active -> Bool ,
        first_name -> Nullable<VarChar>,
        last_name -> Nullable<VarChar>,
        middle_name -> Nullable<VarChar>,
        gender -> Nullable<VarChar>,
        birthdate -> Nullable<Date>,
        avatar -> Nullable<VarChar>,
        last_login_at -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        saga_id -> VarChar,
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Clone)]
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
    pub gender: Gender,
    pub birthdate: Option<NaiveDate>,
    pub avatar: Option<String>,
    pub last_login_at: SystemTime,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub saga_id: String,
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
    pub gender: Gender,
    pub birthdate: Option<NaiveDate>,
    pub last_login_at: SystemTime,
    pub saga_id: String,
}

/// Payload for updating users
#[derive(Debug, Serialize, Deserialize, Insertable, Validate, AsChangeset)]
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
        self.phone.is_none() && self.first_name.is_none() && self.last_name.is_none() && self.middle_name.is_none() && self.gender.is_none()
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
            gender: Gender::Undefined,
            birthdate: None,
            last_login_at: SystemTime::now(),
            saga_id: identity.saga_id,
        }
    }
}
