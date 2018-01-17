use models::schema::users;
use validator::Validate;

/// Payload for creating users
#[derive(Serialize, Deserialize, Insertable, Validate, Clone)]
#[table_name = "users"]
pub struct NewUser {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = "6", max = "30", message = "Password should be between 6 and 30 symbols"))]
    pub password: String,
}

/// Payload for updating users
#[derive(Serialize, Deserialize, Insertable, Validate)]
#[table_name = "users"]
pub struct UpdateUser {
    #[validate(email(message = "Invalid e-mail format"))]
    pub email: String
}
