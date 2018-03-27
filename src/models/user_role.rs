//! Models for managing Roles
use models::{Role, Scope};
use repos::types::DbConnection;
use stq_acl::WithScope;

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

impl WithScope<Scope> for UserRole {
    fn is_in_scope(&self, scope: &Scope, user_id: i32, _conn: Option<&DbConnection>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => self.user_id == user_id,
        }
    }
}
