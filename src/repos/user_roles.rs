//! Repo for user_roles table. UserRole is an entity that connects
//! users and roles. I.e. this table is for user has-many roles
//! relationship

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use stq_acl::*;

use super::error::RepoError as Error;
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
    pub acl: Box<Acl<Resource, Action, Scope, Error, UserRole>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UserRolesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, Error, UserRole>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UserRolesRepo for UserRolesRepoImpl<'a, T> {
    fn list_for_user(&self, user_id_value: i32) -> RepoResult<Vec<Role>> {
        let query = user_roles.filter(user_id.eq(user_id_value));
        query
            .get_results::<UserRole>(self.db_conn)
            .map_err(|e| Error::from(e))
            .and_then(|user_roles_arg| {
                Ok(user_roles_arg
                    .into_iter()
                    .map(|user_role| user_role.role)
                    .collect::<Vec<Role>>())
            })
    }

    fn create(&self, payload: NewUserRole) -> RepoResult<UserRole> {
        let query = diesel::insert_into(user_roles).values(&payload);
        query.get_result(self.db_conn).map_err(Error::from)
    }

    fn delete(&self, payload: OldUserRole) -> RepoResult<UserRole> {
        let filtered = user_roles
            .filter(user_id.eq(payload.user_id))
            .filter(role.eq(payload.role));
        let query = diesel::delete(filtered);
        query.get_result(self.db_conn).map_err(Error::from)
    }

    fn delete_by_user_id(&self, user_id_arg: i32) -> RepoResult<UserRole> {
        let filtered = user_roles.filter(user_id.eq(user_id_arg));
        let query = diesel::delete(filtered);
        query.get_result(self.db_conn).map_err(Error::from)
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
