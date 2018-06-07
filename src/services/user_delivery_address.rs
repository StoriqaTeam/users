//! UserDeliveryAddress Services, presents CRUD operations with user_roles

use futures_cpupool::CpuPool;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use failure::Fail;
use futures::Future;
use r2d2::{ManageConnection, Pool};

use super::types::ServiceFuture;
use errors::ControllerError;
use models::{NewUserDeliveryAddress, UpdateUserDeliveryAddress, UserDeliveryAddress};
use repos::ReposFactory;

pub trait UserDeliveryAddressService {
    /// Returns list of user_delivery_address
    fn get_addresses(&self, user_id: i32) -> ServiceFuture<Vec<UserDeliveryAddress>>;
    /// Create a new user delivery address
    fn create(&self, payload: NewUserDeliveryAddress) -> ServiceFuture<UserDeliveryAddress>;
    /// Update a user delivery address
    fn update(&self, id: i32, payload: UpdateUserDeliveryAddress) -> ServiceFuture<UserDeliveryAddress>;
    /// Delete user delivery address
    fn delete(&self, id: i32) -> ServiceFuture<UserDeliveryAddress>;
}

/// UserDeliveryAddress services, responsible for UserDeliveryAddress-related CRUD operations
pub struct UserDeliveryAddressServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub user_id: Option<i32>,
    pub repo_factory: F,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > UserDeliveryAddressServiceImpl<T, M, F>
{
    pub fn new(db_pool: Pool<M>, cpu_pool: CpuPool, user_id: Option<i32>, repo_factory: F) -> Self {
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
    > UserDeliveryAddressService for UserDeliveryAddressServiceImpl<T, M, F>
{
    /// Returns list of user_delivery_address
    fn get_addresses(&self, user_id: i32) -> ServiceFuture<Vec<UserDeliveryAddress>> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let curent_user_id = self.user_id.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(ControllerError::Connection).into())
                        .and_then(move |conn| {
                            let users_delivery_address_repo = repo_factory.create_users_delivery_address_repo(&*conn, curent_user_id);
                            users_delivery_address_repo.list_for_user(user_id)
                        })
                })
                .map_err(|e| {
                    e.context("Service UserDeliveryAddress, get_addresses endpoint error occured.")
                        .into()
                }),
        )
    }

    /// Delete user delivery address
    fn delete(&self, id: i32) -> ServiceFuture<UserDeliveryAddress> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(ControllerError::Connection).into())
                        .and_then(move |conn| {
                            let users_delivery_address_repo = repo_factory.create_users_delivery_address_repo(&*conn, user_id);
                            users_delivery_address_repo.delete(id)
                        })
                })
                .map_err(|e| e.context("Service UserDeliveryAddress, delete endpoint error occured.").into()),
        )
    }

    /// Create a new user delivery address
    fn create(&self, payload: NewUserDeliveryAddress) -> ServiceFuture<UserDeliveryAddress> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(ControllerError::Connection).into())
                        .and_then(move |conn| {
                            let users_delivery_address_repo = repo_factory.create_users_delivery_address_repo(&*conn, user_id);
                            users_delivery_address_repo.create(payload)
                        })
                })
                .map_err(|e| e.context("Service UserDeliveryAddress, create endpoint error occured.").into()),
        )
    }

    /// Update a user delivery address
    fn update(&self, id: i32, payload: UpdateUserDeliveryAddress) -> ServiceFuture<UserDeliveryAddress> {
        let db_pool = self.db_pool.clone();
        let repo_factory = self.repo_factory.clone();
        let user_id = self.user_id.clone();

        Box::new(
            self.cpu_pool
                .spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| e.context(ControllerError::Connection).into())
                        .and_then(move |conn| {
                            let users_delivery_address_repo = repo_factory.create_users_delivery_address_repo(&*conn, user_id);
                            users_delivery_address_repo.update(id, payload)
                        })
                })
                .map_err(|e| e.context("Service UserDeliveryAddress, update endpoint error occured.").into()),
        )
    }
}
