//! Models for managing Roles
use std::time::SystemTime;

use models::Role;
use schema::user_roles;

#[derive(Serialize, Queryable, Debug)]
pub struct UserRole {
    pub id: i32,
    pub user_id: i32,
    pub role: Role,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, Insertable)]
#[table_name = "user_roles"]
pub struct NewUserRole {
    pub user_id: i32,
    pub role: Role,
}

#[derive(Clone, Debug, Serialize, Deserialize, Insertable)]
#[table_name = "user_roles"]
pub struct OldUserRole {
    pub user_id: i32,
    pub role: Role,
}
