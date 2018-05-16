//! UserRoles Services, presents CRUD operations with user_roles

use futures_cpupool::CpuPool;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use r2d2::{ManageConnection, Pool};

use super::error::ServiceError;
use super::types::ServiceFuture;
use models::{NewUserRole, OldUserRole, Role, UserRole};
use repos::roles_cache::RolesCacheImpl;
use repos::ReposFactory;

pub trait UserRolesService {
    /// Returns role by user ID
    fn get_roles(&self, user_id: i32) -> ServiceFuture<Vec<Role>>;
    /// Delete specific user role
    fn delete(&self, payload: OldUserRole) -> ServiceFuture<UserRole>;
    /// Creates new user_role
    fn create(&self, payload: NewUserRole) -> ServiceFuture<UserRole>;
    /// Deletes default roles for user
    fn delete_default(&self, user_id: i32) -> ServiceFuture<UserRole>;
    /// Creates default roles for user
    fn create_default(&self, user_id: i32) -> ServiceFuture<UserRole>;
}

/// UserRoles services, responsible for UserRole-related CRUD operations
pub struct UserRolesServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub cached_roles: RolesCacheImpl,
    pub repo_factory: F,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > UserRolesServiceImpl<T, M, F>
{
    pub fn new(db_pool: Pool<M>, cpu_pool: CpuPool, cached_roles: RolesCacheImpl, repo_factory: F) -> Self {
        Self {
            db_pool,
            cpu_pool,
            cached_roles,
            repo_factory,
        }
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > UserRolesService for UserRolesServiceImpl<T, M, F>
{
    /// Returns role by user ID
    fn get_roles(&self, user_id: i32) -> ServiceFuture<Vec<Role>> {
        let db_pool = self.db_pool.clone();
        let cached_roles = self.cached_roles.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool.get().map_err(|e| ServiceError::Connection(e.into())).and_then(move |conn| {
                if cached_roles.contains(user_id) {
                    let roles = cached_roles.get(user_id);
                    Ok(roles)
                } else {
                    let user_roles_repo = repo_factory.create_user_roles_repo(&*conn);
                    user_roles_repo
                        .list_for_user(user_id)
                        .map_err(ServiceError::from)
                        .and_then(|roles| {
                            cached_roles.add_roles(user_id, &roles);
                            Ok(roles)
                        })
                }
            })
        }))
    }

    /// Deletes specific user role
    fn delete(&self, payload: OldUserRole) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let cached_roles = self.cached_roles.clone();
        let user_id = payload.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let user_roles_repo = repo_factory.create_user_roles_repo(&*conn);
                    user_roles_repo.delete(payload).map_err(ServiceError::from)
                })
                .and_then(|user_role| {
                    cached_roles.remove(user_id);
                    Ok(user_role)
                })
        }))
    }

    /// Creates new user_role
    fn create(&self, new_user_role: NewUserRole) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let cached_roles = self.cached_roles.clone();
        let user_id = new_user_role.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let user_roles_repo = repo_factory.create_user_roles_repo(&*conn);
                    user_roles_repo.create(new_user_role).map_err(ServiceError::from)
                })
                .and_then(|user_role| {
                    cached_roles.remove(user_id);
                    Ok(user_role)
                })
        }))
    }

    /// Deletes default roles for user
    fn delete_default(&self, user_id_arg: i32) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let cached_roles = self.cached_roles.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let user_roles_repo = repo_factory.create_user_roles_repo(&*conn);
                    user_roles_repo.delete_by_user_id(user_id_arg).map_err(ServiceError::from)
                })
                .and_then(|user_role| {
                    cached_roles.remove(user_id_arg);
                    Ok(user_role)
                })
        }))
    }

    /// Creates default roles for user
    fn create_default(&self, user_id_arg: i32) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let cached_roles = self.cached_roles.clone();
        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| ServiceError::Connection(e.into()))
                .and_then(move |conn| {
                    let defaul_role = NewUserRole {
                        user_id: user_id_arg,
                        role: Role::User,
                    };
                    let user_roles_repo = repo_factory.create_user_roles_repo(&*conn);
                    user_roles_repo.create(defaul_role).map_err(ServiceError::from)
                })
                .and_then(|user_role| {
                    cached_roles.remove(user_id_arg);
                    Ok(user_role)
                })
        }))
    }
}
