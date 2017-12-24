use schema::users;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Insertable, Validate)]
#[table_name = "users"]
pub struct NewUser<'a> {
    #[validate(email)]
    pub email: &'a str,
    #[validate(length(min = "6", max = "30"))]
    pub password: &'a str,
}

#[derive(Debug, Serialize, Deserialize, Insertable, Validate)]
#[table_name = "users"]
pub struct UpdateUser<'a> {
    #[validate(email)]
    pub email: &'a str
}
