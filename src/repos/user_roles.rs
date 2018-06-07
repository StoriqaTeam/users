//! Repo for user_roles table. UserRole is an entity that connects
//! users and roles. I.e. this table is for user has-many roles
//! relationship

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;

use stq_acl::*;

use super::types::RepoResult;
use models::authorization::*;
use models::user_role::user_roles::dsl::*;
use models::{NewUserRole, OldUserRole, Role, UserRole};

/// UserRoles repository for handling UserRoles
pub trait UserRolesRepo {
    /// Returns list of user_roles for a specific user
    fn list_for_user(&self, user_id: i32) -> RepoResult<Vec<Role>>;

    /// Create a new user role
    fn create(&self, payload: NewUserRole) -> RepoResult<UserRole>;

    /// Delete role of a user
    fn delete(&self, payload: OldUserRole) -> RepoResult<UserRole>;

    /// Delete user roles by user id
    fn delete_by_user_id(&self, user_id_arg: i32) -> RepoResult<UserRole>;
}

/// Implementation of UserRoles trait
pub struct UserRolesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, UserRole>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UserRolesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, UserRole>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UserRolesRepo for UserRolesRepoImpl<'a, T> {
    /// Returns list of user_roles for a specific user
    fn list_for_user(&self, user_id_value: i32) -> RepoResult<Vec<Role>> {
        let query = user_roles.filter(user_id.eq(user_id_value));
        query
            .get_results::<UserRole>(self.db_conn)
            .map(|user_roles_arg| user_roles_arg.into_iter().map(|user_role| user_role.role).collect::<Vec<Role>>())
            .map_err(|e| {
                e.context(format!("list of user_roles for user {} error occured", user_id_value))
                    .into()
            })
    }

    /// Create a new user role
    fn create(&self, payload: NewUserRole) -> RepoResult<UserRole> {
        let query = diesel::insert_into(user_roles).values(&payload);
        query
            .get_result(self.db_conn)
            .map_err(|e| e.context(format!("Create a new user role {:?} error occured", payload)).into())
    }

    /// Delete role of a user
    fn delete(&self, payload: OldUserRole) -> RepoResult<UserRole> {
        let filtered = user_roles.filter(user_id.eq(payload.user_id)).filter(role.eq(payload.role.clone()));
        let query = diesel::delete(filtered);
        query
            .get_result(self.db_conn)
            .map_err(|e| e.context(format!("Delete user role {:?} error occured", payload)).into())
    }

    /// Delete user roles by user id
    fn delete_by_user_id(&self, user_id_arg: i32) -> RepoResult<UserRole> {
        let filtered = user_roles.filter(user_id.eq(user_id_arg));
        let query = diesel::delete(filtered);
        query
            .get_result(self.db_conn)
            .map_err(|e| e.context(format!("Delete user {} roles error occured", user_id_arg)).into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, UserRole>
    for UserRolesRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: i32, scope: &Scope, obj: Option<&UserRole>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(user_role) = obj {
                    user_role.user_id == user_id_arg
                } else {
                    false
                }
            }
        }
    }
}
