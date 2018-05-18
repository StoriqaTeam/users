//! Repo for user_delivery_address table. UserDeliveryAddress is an entity that connects
//! users and roles. I.e. this table is for user has-many roles
//! relationship

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use stq_acl::*;

use super::acl;
use super::error::RepoError as Error;
use super::types::RepoResult;
use models::authorization::*;
use models::user_delivery_address::user_delivery_address::dsl::*;
use models::{NewUserDeliveryAddress, UpdateUserDeliveryAddress, UserDeliveryAddress};

/// UserDeliveryAddresss repository for handling UserDeliveryAddresss
pub trait UserDeliveryAddresssRepo {
    /// Returns list of user_delivery_address for a specific user
    fn list_for_user(&self, user_id: i32) -> RepoResult<Vec<UserDeliveryAddress>>;

    /// Create a new user delivery address
    fn create(&self, payload: NewUserDeliveryAddress) -> RepoResult<UserDeliveryAddress>;

    /// Update a user delivery address
    fn update(&self, id: i32, payload: UpdateUserDeliveryAddress) -> RepoResult<UserDeliveryAddress>;

    /// Delete user delivery address
    fn delete(&self, id: i32) -> RepoResult<UserDeliveryAddress>;
}

/// Implementation of UserDeliveryAddresss trait
pub struct UserDeliveryAddresssRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, Error, UserDeliveryAddress>>,
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UserDeliveryAddresssRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, Error, UserDeliveryAddress>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UserDeliveryAddresssRepo
    for UserDeliveryAddresssRepoImpl<'a, T>
{
    fn list_for_user(&self, user_id_value: i32) -> RepoResult<Vec<UserDeliveryAddress>> {
        let query = user_delivery_address
            .filter(user_id.eq(user_id_value))
            .order(id.desc());
        query
            .get_results::<UserDeliveryAddress>(self.db_conn)
            .map_err(Error::from)
            .and_then(|addresses: Vec<UserDeliveryAddress>| {
                for addres in addresses.iter() {
                    acl::check(
                        &*self.acl,
                        &Resource::UserDeliveryAddresses,
                        &Action::Read,
                        self,
                        Some(&addres),
                    )?;
                }
                Ok(addresses)
            })
    }

    fn create(&self, payload: NewUserDeliveryAddress) -> RepoResult<UserDeliveryAddress> {
        let query = diesel::insert_into(user_delivery_address).values(&payload);
        query
            .get_result(self.db_conn)
            .map_err(Error::from)
            .and_then(|addres| {
                acl::check(
                    &*self.acl,
                    &Resource::UserDeliveryAddresses,
                    &Action::Write,
                    self,
                    Some(&addres),
                )?;
                Ok(addres)
            })
            .and_then(|new_address| {
                if new_address.is_priority {
                    // set all other addresses priority to false
                    let filter = user_delivery_address.filter(user_id.eq(new_address.user_id).and(id.ne(new_address.id)));
                    let query = diesel::update(filter).set(is_priority.eq(false));
                    let _ = query
                        .get_result::<UserDeliveryAddress>(self.db_conn)
                        .map_err(Error::from);
                }
                Ok(new_address)
            })
    }

    fn update(&self, id_arg: i32, payload: UpdateUserDeliveryAddress) -> RepoResult<UserDeliveryAddress> {
        let query = user_delivery_address.find(id_arg.clone());

        query
            .get_result(self.db_conn)
            .map_err(Error::from)
            .and_then(|addres: UserDeliveryAddress| {
                acl::check(
                    &*self.acl,
                    &Resource::UserDeliveryAddresses,
                    &Action::Write,
                    self,
                    Some(&addres),
                )
            })
            .and_then(|_| {
                let filter = user_delivery_address.filter(id.eq(id_arg));

                let query = diesel::update(filter).set(&payload);
                query
                    .get_result::<UserDeliveryAddress>(self.db_conn)
                    .map_err(Error::from)
            })
            .and_then(|updated_address| {
                if let Some(is_priority_arg) = payload.is_priority {
                    if is_priority_arg {
                        // set all other addresses priority to false
                        let filter = user_delivery_address.filter(
                            user_id
                                .eq(updated_address.user_id)
                                .and(id.ne(updated_address.id)),
                        );
                        let query = diesel::update(filter).set(is_priority.eq(false));
                        let _ = query
                            .get_result::<UserDeliveryAddress>(self.db_conn)
                            .map_err(Error::from);
                    }
                }
                Ok(updated_address)
            })
    }

    /// Delete user delivery address
    fn delete(&self, id_arg: i32) -> RepoResult<UserDeliveryAddress> {
        let query = user_delivery_address.find(id_arg.clone());

        query
            .get_result(self.db_conn)
            .map_err(Error::from)
            .and_then(|addres: UserDeliveryAddress| {
                acl::check(
                    &*self.acl,
                    &Resource::UserDeliveryAddresses,
                    &Action::Write,
                    self,
                    Some(&addres),
                )
            })
            .and_then(|_| {
                let filtered = user_delivery_address.filter(id.eq(id_arg));
                let query = diesel::delete(filtered);
                query.get_result(self.db_conn).map_err(Error::from)
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, UserDeliveryAddress>
    for UserDeliveryAddresssRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: i32, scope: &Scope, obj: Option<&UserDeliveryAddress>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(addres) = obj {
                    addres.user_id == user_id_arg
                } else {
                    false
                }
            }
        }
    }
}
