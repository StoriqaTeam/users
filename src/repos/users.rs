//! Users repo, presents CRUD operations with db for users
use std::convert::From;

use diesel;
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::select;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use stq_acl::*;

use super::acl;
use super::error::RepoError;
use models::authorization::*;
use models::user::user::users::dsl::*;
use models::{NewUser, UpdateUser, User, UserId};
use stq_acl::*;

/// Users repository, responsible for handling users
pub struct UsersRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, RepoError, User>>,
}

pub trait UsersRepo {
    /// Find specific user by ID
    fn find(&mut self, user_id: UserId) -> Result<User, RepoError>;

    fn email_exists(&self, email_arg: String) -> Result<bool, RepoError>;

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

    fn delete_by_saga_id(&mut self, saga_id_arg: String) -> Result<User, RepoError>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UsersRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, RepoError, User>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UsersRepo for UsersRepoImpl<'a, T> {
    /// Find specific user by ID
    fn find(&mut self, user_id_arg: UserId) -> Result<User, RepoError> {
        let query = users.find(user_id_arg);

        query.get_result(self.db_conn).map_err(RepoError::from).and_then(|user: User| {
            acl::check(&*self.acl, &Resource::Users, &Action::Read, self, Some(&user)).and_then(|_| Ok(user))
        })
    }

    fn email_exists(&self, email_arg: String) -> Result<bool, RepoError> {
        let query = select(exists(users.filter(email.eq(email_arg))));

        query
            .get_result(self.db_conn)
            .map_err(RepoError::from)
            .and_then(|exists| acl::check(&*self.acl, &Resource::Users, &Action::Read, self, None).and_then(|_| Ok(exists)))
    }

    /// Find specific user by email
    fn find_by_email(&mut self, email_arg: String) -> Result<User, RepoError> {
        let query = users.filter(email.eq(email_arg));

        query
            .first::<User>(self.db_conn)
            .map_err(RepoError::from)
            .and_then(|user: User| {
                acl::check(&*self.acl, &Resource::Users, &Action::Read, self, Some(&user)).and_then(|_| Ok(user))
            })
    }

    /// Returns list of users, limited by `from` and `count` parameters
    fn list(&mut self, from: i32, count: i64) -> Result<Vec<User>, RepoError> {
        let query = users.filter(is_active.eq(true)).filter(id.ge(from)).order(id).limit(count);

        query
            .get_results(self.db_conn)
            .map_err(RepoError::from)
            .and_then(|users_res: Vec<User>| {
                for user in users_res.iter() {
                    acl::check(
                        &*self.acl,
                        &Resource::Users,
                        &Action::Read,
                        self,
                        Some(&user),
                    )?;
                }

                Ok(users_res)
            })
    }

    /// Creates new user
    // TODO - set e-mail uniqueness in database
    fn create(&mut self, payload: NewUser) -> Result<User, RepoError> {
        let query_user = diesel::insert_into(users).values(&payload);
        query_user.get_result::<User>(self.db_conn).map_err(RepoError::from)
    }

    /// Updates specific user
    fn update(&mut self, user_id_arg: UserId, payload: UpdateUser) -> Result<User, RepoError> {
        let query = users.find(user_id_arg.clone());

        query
            .get_result(self.db_conn)
            .map_err(RepoError::from)
            .and_then(|user: User| acl::check(&*self.acl, &Resource::Users, &Action::Write, self, Some(&user)))
            .and_then(|_| {
                let filter = users.filter(id.eq(user_id_arg)).filter(is_active.eq(true));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<User>(self.db_conn).map_err(RepoError::from)
            })
    }

    /// Deactivates specific user
    fn deactivate(&mut self, user_id_arg: UserId) -> Result<User, RepoError> {
        let query = users.find(user_id_arg.clone());

        query
            .get_result(self.db_conn)
            .map_err(RepoError::from)
            .and_then(|user: User| acl::check(&*self.acl, &Resource::Users, &Action::Write, self, Some(&user)))
            .and_then(|_| {
                let filter = users.filter(id.eq(user_id_arg)).filter(is_active.eq(true));
                let query = diesel::update(filter).set(is_active.eq(false));

                query.get_result(self.db_conn).map_err(RepoError::from)
            })
    }

    /// Deactivates specific user
    fn delete_by_saga_id(&mut self, saga_id_arg: String) -> Result<User, RepoError> {
        let filtered = users.filter(saga_id.eq(saga_id_arg));
        let query = diesel::delete(filtered);
        query.get_result(self.db_conn).map_err(RepoError::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, User>
for UsersRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: i32, scope: &Scope, obj: Option<&User>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(user) = obj {
                    user.id == UserId(user_id_arg)
                } else {
                    false
                }
            }
        }
    }
}
