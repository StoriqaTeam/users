//! Models for managing Roles

use models::{Role, Scope, WithScope};
use repos::types::DbConnection;

table! {
    user_roles (id) {
        id -> Integer,
        user_id -> Integer,
        role -> VarChar,
    }
}

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "user_roles"]
pub struct UserRole {
    pub id: i32,
    pub user_id: i32,
    pub role: Role,
}

#[derive(Serialize, Deserialize, Insertable, Clone)]
#[table_name = "user_roles"]
pub struct NewUserRole {
    pub user_id: i32,
    pub role: Role,
}

#[derive(Serialize, Deserialize, Insertable, Clone)]
#[table_name = "user_roles"]
pub struct OldUserRole {
    pub user_id: i32,
    pub role: Role,
}

impl WithScope for UserRole {
    fn is_in_scope(&self, scope: &Scope, user_id: i32, _conn: Option<&DbConnection>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => self.user_id == user_id,
        }
    }
}
