//! UserRoles Services, presents CRUD operations with user_roles

use futures_cpupool::CpuPool;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;
use futures::Future;
use r2d2::{ManageConnection, Pool};

use stq_types::{UserId, UsersRole};

use super::types::ServiceFuture;
use errors::Error;
use models::{NewUserRole, OldUserRole, UserRole};
use repos::ReposFactory;

pub trait UserRolesService {
    /// Returns role by user ID
    fn get_roles(&self, user_id: UserId) -> ServiceFuture<Vec<UsersRole>>;
    /// Delete specific user role
    fn delete(&self, payload: OldUserRole) -> ServiceFuture<UserRole>;
    /// Creates new user_role
    fn create(&self, payload: NewUserRole) -> ServiceFuture<UserRole>;
    /// Deletes default roles for user
    fn delete_default(&self, user_id: UserId) -> ServiceFuture<UserRole>;
    /// Creates default roles for user
    fn create_default(&self, user_id: UserId) -> ServiceFuture<UserRole>;
}

/// UserRoles services, responsible for UserRole-related CRUD operations
pub struct UserRolesServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    user_id: Option<UserId>,
    pub repo_factory: F,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > UserRolesServiceImpl<T, M, F>
{
    pub fn new(db_pool: Pool<M>, cpu_pool: CpuPool, user_id: Option<UserId>, repo_factory: F) -> Self {
        Self {
            db_pool,
            cpu_pool,
            user_id,
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
    fn get_roles(&self, user_id: UserId) -> ServiceFuture<Vec<UsersRole>> {
        let db_pool = self.db_pool.clone();
        let current_uid = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let user_roles_repo = repo_factory.create_user_roles_repo(&*conn, current_uid);
                            user_roles_repo.list_for_user(user_id)
                        })
                }).map_err(|e: FailureError| e.context("Service user_roles, get_roles endpoint error occured.").into()),
        )
    }

    /// Deletes specific user role
    fn delete(&self, payload: OldUserRole) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let current_uid = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let user_roles_repo = repo_factory.create_user_roles_repo(&*conn, current_uid);
                            user_roles_repo.delete(payload)
                        })
                }).map_err(|e: FailureError| e.context("Service user_roles, delete endpoint error occured.").into()),
        )
    }

    /// Creates new user_role
    fn create(&self, new_user_role: NewUserRole) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let current_uid = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let user_roles_repo = repo_factory.create_user_roles_repo(&*conn, current_uid);
                            conn.transaction::<UserRole, FailureError, _>(move || user_roles_repo.create(new_user_role))
                        })
                }).map_err(|e: FailureError| e.context("Service user_roles, create endpoint error occured.").into()),
        )
    }

    /// Deletes default roles for user
    fn delete_default(&self, user_id_arg: UserId) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let current_uid = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let user_roles_repo = repo_factory.create_user_roles_repo(&*conn, current_uid);
                            user_roles_repo.delete_by_user_id(user_id_arg)
                        })
                }).map_err(|e: FailureError| e.context("Service user_roles, delete_default endpoint error occured.").into()),
        )
    }

    /// Creates default roles for user
    fn create_default(&self, user_id_arg: UserId) -> ServiceFuture<UserRole> {
        let db_pool = self.db_pool.clone();
        let current_uid = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(Error::Connection).into())
                        .and_then(move |conn| {
                            let defaul_role = NewUserRole {
                                user_id: user_id_arg,
                                role: UsersRole::User,
                            };
                            let user_roles_repo = repo_factory.create_user_roles_repo(&*conn, current_uid);
                            user_roles_repo.create(defaul_role)
                        })
                }).map_err(|e: FailureError| e.context("Service user_roles, create_default endpoint error occured.").into()),
        )
    }
}
