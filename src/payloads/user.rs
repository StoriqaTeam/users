use models::schema::users;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Insertable, Validate)]
#[table_name = "users"]
pub struct NewUser<'a> {
    #[validate(email(message = "Invalid e-mail format"))]
    pub email: &'a str,
    #[validate(length(min = "6", max = "30", message = "Password should be between 6 and 30 symbols"))]
    pub password: &'a str,
}

#[derive(Debug, Serialize, Deserialize, Insertable, Validate)]
#[table_name = "users"]
pub struct UpdateUser<'a> {
    #[validate(email(message = "Invalid e-mail format"))]
    pub email: &'a str
}
