//! Users repo, presents CRUD operations with db for users
use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::select;
use diesel::dsl::exists;
use diesel::query_dsl::RunQueryDsl;

use models::{NewUser, UpdateUser, User, UserId};
use models::user::user::users::dsl::*;
use super::error::RepoError;
use super::types::DbConnection;
use repos::acl::Acl;
use models::authorization::*;

/// Users repository, responsible for handling users
pub struct UsersRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
    pub acl: Box<Acl>,
}

pub trait UsersRepo {
    /// Find specific user by ID
    fn find(&mut self, user_id: UserId) -> Result<User, RepoError>;

    fn email_exists(&mut self, email_arg: String) -> Result<bool, RepoError>;

    /// Find specific user by email
    fn find_by_email(&mut self, email_arg: String) -> Result<User, RepoError>;

    /// Returns list of users, limited by `from` and `count` parameters
    fn list(&mut self, from: i32, count: i64) -> Result<Vec<User>, RepoError>;

    /// Creates new user
    fn create(&mut self, payload: NewUser) -> Result<User, RepoError>;

    /// Updates specific user
    fn update(&mut self, user_id: UserId, payload: UpdateUser) -> Result<User, RepoError>;

    /// Deactivates specific user
    fn deactivate(&mut self, user_id: UserId) -> Result<User, RepoError>;
}

impl<'a> UsersRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection, acl: Box<Acl>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a> UsersRepo for UsersRepoImpl<'a> {
    /// Find specific user by ID
    fn find(&mut self, user_id_arg: UserId) -> Result<User, RepoError> {
        let query = users.find(user_id_arg);

        query
            .get_result(&**self.db_conn)
            .map_err(RepoError::from)
            .and_then(|user: User| acl!([user], self.acl, Resource::Users, Action::Read, Some(self.db_conn)).and_then(|_| Ok(user)))
    }

    fn email_exists(&mut self, email_arg: String) -> Result<bool, RepoError> {
        let query = select(exists(users.filter(email.eq(email_arg))));

        query
            .get_result(&**self.db_conn)
            .map_err(RepoError::from)
            .and_then(|exists| acl!([], self.acl, Resource::Users, Action::Read, Some(self.db_conn)).and_then(|_| Ok(exists)))
    }

    /// Find specific user by email
    fn find_by_email(&mut self, email_arg: String) -> Result<User, RepoError> {
        let query = users.filter(email.eq(email_arg));

        query
            .first::<User>(&**self.db_conn)
            .map_err(RepoError::from)
            .and_then(|user: User| acl!([user], self.acl, Resource::Users, Action::Read, Some(self.db_conn)).and_then(|_| Ok(user)))
    }

    /// Returns list of users, limited by `from` and `count` parameters
    fn list(&mut self, from: i32, count: i64) -> Result<Vec<User>, RepoError> {
        let query = users
            .filter(is_active.eq(true))
            .filter(id.gt(from))
            .order(id)
            .limit(count);

        query
            .get_results(&**self.db_conn)
            .map_err(RepoError::from)
            .and_then(|users_res: Vec<User>| {
                let resources = users_res.iter().map(|user| (user as &WithScope)).collect();
                acl!(resources, self.acl, Resource::Users, Action::Read, Some(self.db_conn)).and_then(|_| Ok(users_res.clone()))
            })
    }

    /// Creates new user
    // TODO - set e-mail uniqueness in database
    fn create(&mut self, payload: NewUser) -> Result<User, RepoError> {
        let query_user = diesel::insert_into(users).values(&payload);
        query_user
            .get_result::<User>(&**self.db_conn)
            .map_err(RepoError::from)
    }

    /// Updates specific user
    fn update(&mut self, user_id_arg: UserId, payload: UpdateUser) -> Result<User, RepoError> {
        let query = users.find(user_id_arg.clone());

        query
            .get_result(&**self.db_conn)
            .map_err(RepoError::from)
            .and_then(|user: User| acl!([user], self.acl, Resource::Users, Action::Write, Some(self.db_conn)))
            .and_then(|_| {
                let filter = users.filter(id.eq(user_id_arg)).filter(is_active.eq(true));

                let query = diesel::update(filter).set(&payload);
                query
                    .get_result::<User>(&**self.db_conn)
                    .map_err(RepoError::from)
            })
    }

    /// Deactivates specific user
    fn deactivate(&mut self, user_id_arg: UserId) -> Result<User, RepoError> {
        let query = users.find(user_id_arg.clone());

        query
            .get_result(&**self.db_conn)
            .map_err(RepoError::from)
            .and_then(|user: User| acl!([user], self.acl, Resource::Users, Action::Write, Some(self.db_conn)))
            .and_then(|_| {
                let filter = users.filter(id.eq(user_id_arg)).filter(is_active.eq(true));
                let query = diesel::update(filter).set(is_active.eq(false));

                query.get_result(&**self.db_conn).map_err(RepoError::from)
            })
    }
}
