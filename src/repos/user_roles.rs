//! Repo for user_roles table. UserRole is an entity that connects
//! users and roles. I.e. this table is for user has-many roles
//! relationship

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use futures_cpupool::CpuPool;

use models::user_role::user_roles::dsl::*;
use models::{NewUserRole, UserRole};
use models::Role;
use super::error::RepoError;
use super::types::{RepoFuture, DbConnection, DbPool};

/// UserRoles repository for handling UserRoles
pub trait UserRolesRepo {
    /// Returns list of user_roles for a specific user
    fn list_for_user(&self, user_id: i32) -> RepoFuture<Vec<UserRole>>;

    /// Create a new user role
    fn create(&self, payload: NewUserRole) -> RepoFuture<UserRole>;

    /// Delete role of a user
    fn delete(&self, user_id: i32, role: Role) -> RepoFuture<UserRole>;
}

/// Implementation of UserRoles trait
#[derive(Clone)]
pub struct UserRolesRepoImpl {
    pub r2d2_pool: DbPool,
    pub cpu_pool: CpuPool
}

impl UserRolesRepoImpl {
    fn get_connection(&self) -> DbConnection {
        match self.r2d2_pool.get() {
            Ok(connection) => connection,
            Err(e) => panic!("Error obtaining connection from pool: {}", e),
        }
    }

    pub fn new(r2d2_pool: DbPool, cpu_pool: CpuPool) -> Self {
        Self {
            r2d2_pool,
            cpu_pool
        }
    }
}

impl UserRolesRepo for UserRolesRepoImpl {
    fn list_for_user(&self, user_id_value: i32) -> RepoFuture<Vec<UserRole>> {
        let conn = self.get_connection();

        Box::new(
            self.cpu_pool.spawn_fn(move || {
                let query = user_roles.filter(id.eq(user_id_value));
                query.get_results(&*conn).map_err(|e| RepoError::from(e))
            })
        )
    }

    fn create(&self, payload: NewUserRole) -> RepoFuture<UserRole> {
        let conn = self.get_connection();

        Box::new(self.cpu_pool.spawn_fn(move || {
            let query = diesel::insert_into(user_roles).values(&payload);
            query.get_result(&*conn).map_err(RepoError::from)
        }))
    }

    fn delete(&self, user_id_value: i32, role_value: Role) -> RepoFuture<UserRole> {
        let conn = self.get_connection();

        Box::new(self.cpu_pool.spawn_fn(move || {
            let filtered = user_roles.filter(user_id.eq(user_id_value)).filter(role.eq(role_value));
            let query = diesel::delete(filtered);
            query.get_result(&*conn).map_err(RepoError::from)
        }))
    }
}
