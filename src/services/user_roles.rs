//! UserRoles Services, presents CRUD operations with user_roles

use futures_cpupool::CpuPool;

use models::{NewUserRole, OldUserRole, UserRole};
use super::types::ServiceFuture;
use super::error::ServiceError;
use repos::types::DbPool;
use repos::acl::SystemACL;
use repos::user_roles::{UserRolesRepo, UserRolesRepoImpl};

pub trait UserRolesService {
    /// Returns user_role by ID
    fn get(&self, user_role_id: i32) -> ServiceFuture<Vec<UserRole>>;
    /// Delete specific user role
    fn delete(&self, payload: OldUserRole) -> ServiceFuture<UserRole>;
    /// Creates new user_role
    fn create(&self, payload: NewUserRole) -> ServiceFuture<UserRole>;
}

/// UserRoles services, responsible for UserRole-related CRUD operations
pub struct UserRolesServiceImpl {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
}

impl UserRolesServiceImpl {
    pub fn new(db_pool: DbPool, cpu_pool: CpuPool) -> Self {
        Self { db_pool, cpu_pool }
    }
}

impl UserRolesService for UserRolesServiceImpl {
    /// Returns user_role by ID
    fn get(&self, user_role_id: i32) -> ServiceFuture<Vec<UserRole>> {
        let db_pool = self.db_pool.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let user_roles_repo = UserRolesRepoImpl::new(&conn, Box::new(SystemACL::new()));
                    user_roles_repo
                        .list_for_user(user_role_id)
                        .map_err(ServiceError::from)
                })
        }))
    }

    /// Deletes specific user role
    fn delete(&self, payload: OldUserRole) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let user_roles_repo = UserRolesRepoImpl::new(&conn, Box::new(SystemACL::new()));
                    user_roles_repo.delete(payload).map_err(ServiceError::from)
                })
        }))
    }

    /// Creates new user_role
    fn create(&self, new_user_role: NewUserRole) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let user_roles_repo = UserRolesRepoImpl::new(&conn, Box::new(SystemACL::new()));
                    user_roles_repo
                        .create(new_user_role)
                        .map_err(ServiceError::from)
                })
        }))
    }
}
