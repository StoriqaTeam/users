//! Users repo, presents CRUD operations with db for users
use std::convert::From;
use std::sync::Arc;

use diesel;
use diesel::prelude::*;
use diesel::select;
use diesel::dsl::exists;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::pg::PgConnection;

use models::{UpdateUser, User, NewUser, UserId};
use models::user::user::users::dsl::*;
use super::error::Error;
use super::types::DbConnection;
use repos::acl::Acl;

/// Users repository, responsible for handling users
pub struct UsersRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
    pub acl : Arc<Acl>
}

pub trait UsersRepo {
    /// Find specific user by ID
    fn find(&self, user_id: UserId) -> Result<User, Error>;

    fn email_exists(&self, email_arg: String) -> Result<bool, Error>;

    /// Find specific user by email
    fn find_by_email(&self, email_arg: String) -> Result<User, Error>;

    /// Returns list of users, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> Result<Vec<User>, Error>;

    /// Creates new user
    fn create(&self, payload: NewUser) -> Result<User, Error>;

    /// Updates specific user
    fn update(&self, user_id: UserId, payload: UpdateUser) -> Result<User, Error>;

    /// Deactivates specific user
    fn deactivate(&self, user_id: UserId) -> Result<User, Error>;
}

impl<'a> UsersRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection, acl : Arc<Acl>) -> Self {
        Self {
            db_conn,
            acl
        }
    }
}

impl<'a> UsersRepo for UsersRepoImpl<'a> {
    /// Find specific user by ID
    fn find(&self, user_id_arg: UserId) -> Result<User, Error> {
        let query = users.find(user_id_arg);

        query.get_result(&**self.db_conn).map_err(Error::from)
    }

    fn email_exists(&self, email_arg: String) -> Result<bool, Error> {
        let query = select(exists(users.filter(email.eq(email_arg))));

        query.get_result(&**self.db_conn).map_err(Error::from)
    }

    /// Find specific user by email
    fn find_by_email(&self, email_arg: String) -> Result<User, Error> {
        let query = users
            .filter(email.eq(email_arg));

        query.first::<User>(&**self.db_conn).map_err(|e| Error::from(e))
    }

    /// Returns list of users, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> Result<Vec<User>, Error> {
        let query = users
            .filter(is_active.eq(true))
            .filter(id.gt(from))
            .order(id)
            .limit(count);

        query.get_results(&**self.db_conn).map_err(|e| Error::from(e))
    }

    /// Creates new user
    // TODO - set e-mail uniqueness in database
    fn create(&self, payload: NewUser) -> Result<User, Error> {
        let query_user = diesel::insert_into(users).values(&payload);
        query_user
            .get_result::<User>(&**self.db_conn)
            .map_err(Error::from)
    }

    /// Updates specific user
    fn update(&self, user_id_arg: UserId, payload: UpdateUser) -> Result<User, Error> {
        let filter = users.filter(id.eq(user_id_arg)).filter(is_active.eq(true));

        let query = diesel::update(filter).set(&payload);
        query.get_result::<User>(&**self.db_conn).map_err(|e| Error::from(e))
    }

    /// Deactivates specific user
    fn deactivate(&self, user_id_arg: UserId) -> Result<User, Error> {
        let filter = users.filter(id.eq(user_id_arg)).filter(is_active.eq(true));
        let query = diesel::update(filter).set(is_active.eq(false));

        query.get_result(&**self.db_conn).map_err(Error::from)
    }
}
