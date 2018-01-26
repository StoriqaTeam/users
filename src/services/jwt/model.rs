use std::time::SystemTime;
use std::str::FromStr;
use std::str;

use models::user::{UpdateUser, Gender, User};


#[derive(Serialize, Deserialize, Clone)]
pub struct GoogleProfile {
  pub family_name: String,
  pub name: String,
  pub picture: String,
  pub email: String,
  pub given_name: String,
  pub id: String,
  pub hd: String,
  pub verified_email: bool
}

impl From<GoogleProfile> for UpdateUser {
    fn from(google_id: GoogleProfile) -> Self {
        UpdateUser {
            email: google_id.email,
            phone: None,
            first_name: Some(google_id.name),
            last_name: Some(google_id.family_name),
            middle_name:  None,
            gender: Gender::Undefined,
            birthdate: None,
            last_login_at: SystemTime::now(),
        }
    }
}


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

pub trait IntoUser {
    fn merge_into_user(&self, user: User) -> UpdateUser;
}

impl IntoUser for FacebookProfile {
    fn merge_into_user(&self, user: User) -> UpdateUser {
        let first_name = if user.first_name.is_none() {
            Some(self.first_name.clone())
        } else {
            user.first_name
        };
        let last_name = if user.last_name.is_none() {
            Some(self.last_name.clone())
        } else {
            user.last_name
        };
        let gender = if user.gender == Gender::Undefined {
            Gender::from_str(self.gender.as_ref()).unwrap()
        } else {
            user.gender
        };
        UpdateUser {
            email: user.email,
            phone: user.phone,
            first_name: first_name,
            last_name: last_name,
            middle_name:  user.middle_name,
            gender: gender,
            birthdate: user.birthdate,
            last_login_at: SystemTime::now(),
        }
    }
}

impl IntoUser for GoogleProfile {
    fn merge_into_user(&self, user: User) -> UpdateUser {
        let first_name = if user.first_name.is_none() {
            Some(self.name.clone())
        } else {
            user.first_name
        };
        let last_name = if user.last_name.is_none() {
            Some(self.family_name.clone())
        } else {
            user.last_name
        };
        UpdateUser {
            email: user.email,
            phone: user.phone,
            first_name: first_name,
            last_name: last_name,
            middle_name:  user.middle_name,
            gender: user.gender,
            birthdate: user.birthdate,
            last_login_at: SystemTime::now(),
        }
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub struct FacebookProfile {
    pub id: String,
    pub email: String,
    pub gender: String,
    pub first_name: String,
    pub last_name: String,
    pub name: String,
}

impl From<FacebookProfile> for UpdateUser {
    fn from(facebook_id: FacebookProfile) -> Self {
        UpdateUser {
            email: facebook_id.email,
            phone: None,
            first_name: Some(facebook_id.first_name),
            last_name: Some(facebook_id.last_name),
            middle_name:  None,
            gender: Gender::from_str(facebook_id.gender.as_ref()).unwrap(),
            birthdate: None,
            last_login_at: SystemTime::now(),
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct JWTPayload {
    pub user_email: String,
}

impl JWTPayload {
    pub fn new<S: Into<String>>(email: S) -> Self {
        Self {
            user_email: email.into(),
        }
    }
}

