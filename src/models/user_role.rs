use validator::Validate;

use super::authorization::Role;
use models::schema::user_roles;

#[derive(Queryable, Insertable, Debug)]
#[table_name = "user_roles"]
pub struct UserRole {
    pub id: i32,
    pub user_id: i32,
    pub role: Role
}

#[derive(Serialize, Deserialize, Insertable, Validate, Clone)]
#[table_name = "user_roles"]
pub struct NewUserRole {
    pub user_id: i32,
    pub role: Role
}

