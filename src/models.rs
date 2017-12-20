use schema::users;
use validator::Validate;

#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub is_active: bool,
}

#[derive(Debug, Validate, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    #[validate(email)]
    pub email: &'a str,
    #[validate(length(min = "6", max = "20"))]
    pub password: &'a str,
}

#[derive(Debug, Validate, Deserialize, Insertable)]
#[table_name = "users"]
pub struct UpdateUser<'a> {
    #[validate(email)]
    pub email: &'a str
}
