use std::time::SystemTime;
use std::error::Error;
use std::fmt;

use diesel::pg::Pg;
use diesel::types::FromSqlRow;
use diesel::types::{Text};
use diesel::row::Row;
use diesel::expression::AsExpression;
use diesel::dsl::AsExprOf;
use diesel::Queryable;

use validator::Validate;
use models::schema::users;
use models::schema::identity;


#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)] 
pub enum Provider {
   Email,
   UnverifiedEmail,
   Facebook,
   Google
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Provider::Email => "email",
            Provider::UnverifiedEmail => "unverifiedemail",
            Provider::Facebook => "facebook",
            Provider::Google => "google",
        })
    }
}

impl FromSqlRow<Text, Pg> for Provider {
    fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error+Send+Sync>> {
        match String::build_from_row(row)?.as_ref() {
            "email" => Ok(Provider::Email),
            "unverifiedemail" => Ok(Provider::UnverifiedEmail),
            "facebook" => Ok(Provider::Facebook),
            "google" => Ok(Provider::Google),
            v => Err(format!("Unknown value {} for State found", v).into()),
        }
    }
}

impl AsExpression<Text> for Provider {
    type Expression = AsExprOf<String, Text>;
    fn as_expression(self) -> Self::Expression {
        <String as AsExpression<Text>>::as_expression(self.to_string())
    }
}

impl<'a> AsExpression<Text> for &'a Provider {
    type Expression = AsExprOf<String, Text>;
    fn as_expression(self) -> Self::Expression {
        <String as AsExpression<Text>>::as_expression(self.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)] 
pub enum Gender {
   Male,
   Female,
   Undefined
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Gender::Male => "male",
            Gender::Female => "female",
            Gender::Undefined => "undefined",
        })
    }
}

impl FromSqlRow<Text, Pg> for Gender {
    fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error+Send+Sync>> {
        match String::build_from_row(row)?.as_ref() {
            "male" => Ok(Gender::Male),
            "female" => Ok(Gender::Female),
            "undefined" => Ok(Gender::Undefined),
            v => Err(format!("Unknown value {} for Gender found", v).into()),
        }
    }
}

impl AsExpression<Text> for Gender {
    type Expression = AsExprOf<String, Text>;
    fn as_expression(self) -> Self::Expression {
        <String as AsExpression<Text>>::as_expression(self.to_string())
    }
}

impl<'a> AsExpression<Text> for &'a Gender {
    type Expression = AsExprOf<String, Text>;
    fn as_expression(self) -> Self::Expression {
        <String as AsExpression<Text>>::as_expression(self.to_string())
    }
}


/// Payload for creating identity for users
#[derive(Debug, Serialize, Deserialize, Validate, Insertable)]
#[table_name = "identity"]
pub struct Identity
{
    pub user_id: i32,
    #[validate(email(message = "Invalid email format"))]
    pub user_email: String,
    #[validate(length(min = "6", max = "30", message = "Password should be between 6 and 30 symbols"))]
    pub user_password: Option<String>,
    pub provider: Provider
}


impl Queryable<identity::SqlType, Pg> for Identity {
    type Row = (i32, String, Option<String>, String);

    fn build(row: Self::Row) -> Self {
        Identity {
            user_id: row.0,
            user_email: row.1,
            user_password: row.2,
            provider: match row.3.as_ref() {
                "email" => Provider::Email,
                "unverifiedemail" => Provider::UnverifiedEmail,
                "facebook" => Provider::Facebook,
                "google" => Provider::Google,
                n => panic!("unknown kind: {}", n),
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
   pub id: i32,
   pub email: String,
   pub email_verified: bool,
   pub phone: Option<String>,
   pub phone_verified: bool,
   pub is_active: bool ,
   pub first_name: Option<String>,
   pub last_name: Option<String>,
   pub middle_name: Option<String>,
   pub gender: Gender,
   pub birthdate: Option<SystemTime>,
   pub last_login_at: SystemTime, 
   pub created_at: SystemTime, 
   pub updated_at: SystemTime, 
}

impl Queryable<users::SqlType, Pg> for User {
    type Row = (i32, String, bool, Option<String>, bool, bool, Option<String>, Option<String>, Option<String>, String, Option<SystemTime>, SystemTime,SystemTime,SystemTime);

    fn build(row: Self::Row) -> Self {
        User {
            id: row.0,          
            email: row.1,           
            email_verified: row.2,          
            phone: row.3,           
            phone_verified: row.4,          
            is_active: row.5 ,          
            first_name: row.6,          
            last_name: row.7,           
            middle_name: row.8,         
            gender: match row.9.as_ref() {          
                "male" => Gender::Male,         
                "female" => Gender::Female,         
                "undefined" => Gender::Undefined,           
                n => panic!("unknown gender: {}", n),           
            },          
            birthdate: row.10,          
            last_login_at: row.11,          
            created_at: row.12,         
            updated_at: row.13,         
        }
    }
}

/// Payload for creating users
#[derive(Serialize, Deserialize, Validate, Clone)]
pub struct NewUser {
    #[validate(email(message = "Invalid email format"))]
    pub user_email: String,
    #[validate(length(min = "6", max = "30", message = "Password should be between 6 and 30 symbols"))]
    pub user_password: String,
    pub provider: Provider,
}

/// Payload for updating users
#[derive(Serialize, Deserialize, Insertable, Validate)]
#[table_name = "users"]
pub struct UpdateUser {
    #[validate(email(message = "Invalid e-mail format"))]
    pub email: String,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub gender: Gender,
    pub birthdate: Option<SystemTime>,
    pub last_login_at: SystemTime, 
}

impl From<NewUser> for UpdateUser {
    fn from(new_user: NewUser) -> Self {
        UpdateUser {
            email: new_user.user_email,
            phone: None,
            first_name: None,
            last_name: None,
            middle_name:  None,
            gender: Gender::Undefined,
            birthdate: None,
            last_login_at: SystemTime::now(),
        }
    }
}