//! Repo for user_roles table. UserRole is an entity that connects
//! users and roles. I.e. this table is for user has-many roles
//! relationship

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;

use models::user_role::user_roles::dsl::*;
use models::{NewUserRole, OldUserRole, Role, UserRole};
use super::acl::BoxedAcl;
use super::error::RepoError as Error;
use super::types::{DbConnection, RepoResult};

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
pub struct UserRolesRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
    pub acl: BoxedAcl,
}

impl<'a> UserRolesRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection, acl: BoxedAcl) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a> UserRolesRepo for UserRolesRepoImpl<'a> {
    fn list_for_user(&self, user_id_value: i32) -> RepoResult<Vec<Role>> {
        let query = user_roles.filter(user_id.eq(user_id_value));
        query
            .get_results::<UserRole>(&**self.db_conn)
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
        query.get_result(&**self.db_conn).map_err(Error::from)
    }

    fn delete(&self, payload: OldUserRole) -> RepoResult<UserRole> {
        let filtered = user_roles
            .filter(user_id.eq(payload.user_id))
            .filter(role.eq(payload.role));
        let query = diesel::delete(filtered);
        query.get_result(&**self.db_conn).map_err(Error::from)
    }

    fn delete_by_user_id(&self, user_id_arg: i32) -> RepoResult<UserRole> {
        let filtered = user_roles.filter(user_id.eq(user_id_arg));
        let query = diesel::delete(filtered);
        query.get_result(&**self.db_conn).map_err(Error::from)
    }
}
