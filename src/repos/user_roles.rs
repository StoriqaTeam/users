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
use std::sync::Arc;
use stq_cache::cache::Cache;
use stq_types::{RoleId, UserId, UsersRole};

use repos::legacy_acl::*;

use super::acl;
use super::types::RepoResult;
use models::authorization::*;
use models::{NewUserRole, UserRole};
use repos::acl::RolesCacheImpl;
use schema::user_roles::dsl::*;

/// UserRoles repository for handling UserRoles
pub trait UserRolesRepo {
    /// Returns list of user_roles for a specific user
    fn list_for_user(&self, user_id: UserId) -> RepoResult<Vec<UsersRole>>;

    /// Create a new user role
    fn create(&self, payload: NewUserRole) -> RepoResult<UserRole>;

    /// Delete role of a user
    fn delete_user_role(&self, user_id_arg: UserId, name_arg: UsersRole) -> RepoResult<UserRole>;

    /// Delete role of a user
    fn delete_by_id(&self, id_arg: RoleId) -> RepoResult<UserRole>;

    /// Delete user roles by user id
    fn delete_by_user_id(&self, user_id_arg: UserId) -> RepoResult<Vec<UserRole>>;
}

/// Implementation of UserRoles trait
pub struct UserRolesRepoImpl<'a, C, T>
where
    C: Cache<Vec<UsersRole>>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, UserRole>>,
    pub db_conn: &'a T,
    pub cached_roles: Arc<RolesCacheImpl<C>>,
}

impl<'a, C, T> UserRolesRepoImpl<'a, C, T>
where
    C: Cache<Vec<UsersRole>>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    pub fn new(
        db_conn: &'a T,
        acl: Box<Acl<Resource, Action, Scope, FailureError, UserRole>>,
        cached_roles: Arc<RolesCacheImpl<C>>,
    ) -> Self {
        Self {
            db_conn,
            acl,
            cached_roles,
        }
    }
}

impl<'a, C, T> UserRolesRepo for UserRolesRepoImpl<'a, C, T>
where
    C: Cache<Vec<UsersRole>>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    /// Returns list of user_roles for a specific user
    fn list_for_user(&self, user_id_value: UserId) -> RepoResult<Vec<UsersRole>> {
        debug!("list user roles for id {}.", user_id_value);
        if let Some(roles) = self.cached_roles.get(user_id_value) {
            Ok(roles)
        } else {
            let query = user_roles.filter(user_id.eq(user_id_value));
            query
                .get_results::<UserRole>(self.db_conn)
                .map_err(From::from)
                .and_then(|user_roles_arg: Vec<UserRole>| {
                    for user_role_arg in &user_roles_arg {
                        acl::check(&*self.acl, Resource::UserRoles, Action::Read, self, Some(&user_role_arg))?;
                    }
                    let roles = user_roles_arg
                        .into_iter()
                        .map(|user_role| user_role.name)
                        .collect::<Vec<UsersRole>>();
                    Ok(roles)
                }).and_then(|roles| {
                    if !roles.is_empty() {
                        self.cached_roles.set(user_id_value, roles.clone());
                    }
                    Ok(roles)
                }).map_err(|e: FailureError| {
                    e.context(format!("List user roles for user {} error occured.", user_id_value))
                        .into()
                })
        }
    }

    /// Create a new user role
    fn create(&self, payload: NewUserRole) -> RepoResult<UserRole> {
        self.cached_roles.remove(payload.user_id);
        let query = diesel::insert_into(user_roles).values(&payload);
        query
            .get_result(self.db_conn)
            .map_err(From::from)
            .and_then(|user_role_arg: UserRole| {
                acl::check(&*self.acl, Resource::UserRoles, Action::Create, self, Some(&user_role_arg))?;
                Ok(user_role_arg)
            }).map_err(|e: FailureError| e.context(format!("Create a new user role {:?} error occured", payload)).into())
    }

    /// Delete role of a user
    fn delete_by_id(&self, id_arg: RoleId) -> RepoResult<UserRole> {
        let filtered = user_roles.filter(id.eq(id_arg));
        let query = diesel::delete(filtered);
        query
            .get_result(self.db_conn)
            .map_err(From::from)
            .and_then(|user_role_arg: UserRole| {
                acl::check(&*self.acl, Resource::UserRoles, Action::Delete, self, Some(&user_role_arg))?;
                Ok(user_role_arg)
            }).map(|user_role: UserRole| {
                self.cached_roles.remove(user_role.user_id);
                user_role
            }).map_err(|e: FailureError| e.context(format!("Delete user role {:?} error occured", id_arg)).into())
    }

    /// Delete user roles by user id
    fn delete_by_user_id(&self, user_id_arg: UserId) -> RepoResult<Vec<UserRole>> {
        self.cached_roles.remove(user_id_arg);
        let filtered = user_roles.filter(user_id.eq(user_id_arg));
        let query = diesel::delete(filtered);
        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|user_roles_arg: Vec<UserRole>| {
                for user_role_arg in &user_roles_arg {
                    acl::check(&*self.acl, Resource::UserRoles, Action::Delete, self, Some(&user_role_arg))?;
                }
                Ok(user_roles_arg)
            }).map_err(|e: FailureError| e.context(format!("Delete user {} roles error occured", user_id_arg)).into())
    }

    /// Delete user roles by user id and name
    fn delete_user_role(&self, user_id_arg: UserId, name_arg: UsersRole) -> RepoResult<UserRole> {
        self.cached_roles.remove(user_id_arg);
        let filtered = user_roles.filter(user_id.eq(user_id_arg)).filter(name.eq(name_arg));
        let query = diesel::delete(filtered);
        query
            .get_result(self.db_conn)
            .map_err(From::from)
            .and_then(|user_role_arg| {
                acl::check(&*self.acl, Resource::UserRoles, Action::Delete, self, Some(&user_role_arg))?;
                Ok(user_role_arg)
            }).map_err(|e: FailureError| {
                e.context(format!("Delete user {} role {:?} error occured", user_id_arg, name_arg))
                    .into()
            })
    }
}

impl<'a, C, T> CheckScope<Scope, UserRole> for UserRolesRepoImpl<'a, C, T>
where
    C: Cache<Vec<UsersRole>>,
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
{
    fn is_in_scope(&self, user_id_arg: UserId, scope: &Scope, obj: Option<&UserRole>) -> bool {
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
