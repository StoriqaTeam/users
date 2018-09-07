//! Models for managing profiles from google and facebook
use std::str;
use std::str::FromStr;
use std::time::SystemTime;

use models::{Gender, NewUser, UpdateUser, User};

use uuid::Uuid;

/// User profile from google
#[derive(Serialize, Deserialize, Clone)]
pub struct GoogleProfile {
    pub family_name: String,
    pub name: String,
    pub picture: String,
    pub email: String,
    pub given_name: String,
    pub verified_email: bool,
}

impl From<GoogleProfile> for NewUser {
    fn from(google_id: GoogleProfile) -> Self {
        NewUser {
            email: google_id.email,
            phone: None,
            first_name: Some(google_id.given_name),
            last_name: Some(google_id.family_name),
            middle_name: None,
            gender: Gender::Undefined,
            birthdate: None,
            last_login_at: SystemTime::now(),
            saga_id: Uuid::new_v4().to_string(),
        }
    }
}

/// User profile from facebook
#[derive(Serialize, Deserialize, Clone)]
pub struct FacebookProfile {
    pub id: String,
    pub email: String,
    pub gender: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub name: String,
}

impl From<FacebookProfile> for NewUser {
    fn from(facebook_id: FacebookProfile) -> Self {
        let gender = if let Some(gender) = facebook_id.gender {
            Gender::from_str(&gender).unwrap_or(Gender::Undefined)
        } else {
            Gender::Undefined
        };
        NewUser {
            email: facebook_id.email,
            phone: None,
            first_name: Some(facebook_id.first_name),
            last_name: Some(facebook_id.last_name),
            middle_name: None,
            gender,
            birthdate: None,
            last_login_at: SystemTime::now(),
            saga_id: Uuid::new_v4().to_string(),
        }
    }
}

/// Email trait implemented by Google and Facebook profiles
pub trait Email {
    fn get_email(&self) -> String;
}

impl Email for FacebookProfile {
    fn get_email(&self) -> String {
        self.email.clone()
    }
}

impl Email for GoogleProfile {
    fn get_email(&self) -> String {
        self.email.clone()
    }
}

/// IntoUser trait for merging info from Google and Facebook profiles in users profile in db
pub trait IntoUser {
    fn merge_into_user(&self, user: User) -> UpdateUser;
}

impl IntoUser for FacebookProfile {
    fn merge_into_user(&self, user: User) -> UpdateUser {
        let first_name = if user.first_name.is_none() {
            Some(self.first_name.clone())
        } else {
            None
        };
        let last_name = if user.last_name.is_none() {
            Some(self.last_name.clone())
        } else {
            None
        };
        let gender = if user.gender == Gender::Undefined {
            self.gender.clone().map(|g| Gender::from_str(&g).unwrap_or(Gender::Undefined))
        } else {
            None
        };
        UpdateUser {
            phone: None,
            first_name,
            last_name,
            middle_name: None,
            gender,
            birthdate: None,
            avatar: None,
            is_active: Some(true),
            email_verified: None,
        }
    }
}

impl IntoUser for GoogleProfile {
    fn merge_into_user(&self, user: User) -> UpdateUser {
        let first_name = user.first_name.unwrap_or_else(|| self.given_name.clone());
        let last_name = user.last_name.unwrap_or_else(|| self.family_name.clone());
        UpdateUser {
            phone: None,
            first_name: Some(first_name),
            last_name: Some(last_name),
            middle_name: None,
            gender: None,
            birthdate: None,
            avatar: None,
            is_active: Some(true),
            email_verified: None,
        }
    }
}
