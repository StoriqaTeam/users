//! Users repo, presents CRUD operations with db for users
use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::dsl::exists;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::select;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;

use repos::legacy_acl::*;

use super::acl;
use super::types::RepoResult;
use models::authorization::*;
use models::{NewUser, UpdateUser, User, UserId};
use schema::users::dsl::*;

/// Users repository, responsible for handling users
pub struct UsersRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, User>>,
}

pub trait UsersRepo {
    /// Find specific user by ID
    fn find(&self, user_id: UserId) -> RepoResult<Option<User>>;

    /// Check that user with specified email already exists
    fn email_exists(&self, email_arg: String) -> RepoResult<bool>;

    /// Find specific user by email
    fn find_by_email(&self, email_arg: String) -> RepoResult<Option<User>>;

    /// Returns list of users, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoResult<Vec<User>>;

    /// Creates new user
    fn create(&self, payload: NewUser) -> RepoResult<User>;

    /// Updates specific user
    fn update(&self, user_id: UserId, payload: UpdateUser) -> RepoResult<User>;

    /// Deactivates specific user
    fn deactivate(&self, user_id: UserId) -> RepoResult<User>;

    /// Deletes specific user
    fn delete_by_saga_id(&self, saga_id_arg: String) -> RepoResult<User>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UsersRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, User>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UsersRepo for UsersRepoImpl<'a, T> {
    /// Find specific user by ID
    fn find(&self, user_id_arg: UserId) -> RepoResult<Option<User>> {
        let query = users.find(user_id_arg.clone());

        query
            .get_result(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|user: Option<User>| {
                if let Some(ref user) = user {
                    acl::check(&*self.acl, Resource::Users, Action::Read, self, Some(user))?;
                };
                Ok(user)
            })
            .map_err(|e: FailureError| e.context(format!("Find specific user {} error occured", user_id_arg)).into())
    }

    /// Check that user with specified email already exists
    fn email_exists(&self, email_arg: String) -> RepoResult<bool> {
        let query = select(exists(users.filter(email.eq(email_arg.clone()))));

        query
            .get_result(self.db_conn)
            .map_err(From::from)
            .and_then(|exists| acl::check(&*self.acl, Resource::Users, Action::Read, self, None).and_then(|_| Ok(exists)))
            .map_err(|e: FailureError| {
                e.context(format!("Check that user with email {} already exists error occured", email_arg))
                    .into()
            })
    }

    /// Find specific user by email
    fn find_by_email(&self, email_arg: String) -> RepoResult<Option<User>> {
        let query = users.filter(email.eq(email_arg.clone()));

        query
            .first(self.db_conn)
            .optional()
            .map_err(From::from)
            .and_then(|user: Option<User>| {
                if let Some(ref user) = user {
                    acl::check(&*self.acl, Resource::Users, Action::Read, self, Some(user))?;
                };
                Ok(user)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Find specific user by email {:?} error occured", email_arg))
                    .into()
            })
    }

    /// Returns list of users, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoResult<Vec<User>> {
        let query = users.filter(is_active.eq(true)).filter(id.ge(from)).order(id).limit(count);

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|users_res: Vec<User>| {
                for user in &users_res {
                    acl::check(&*self.acl, Resource::Users, Action::Read, self, Some(&user))?;
                }

                Ok(users_res)
            })
            .map_err(|e: FailureError| {
                e.context(format!("list of users, limited by {} and {} error occured", from, count))
                    .into()
            })
    }

    /// Creates new user
    fn create(&self, payload: NewUser) -> RepoResult<User> {
        let query_user = diesel::insert_into(users).values(&payload);
        query_user
            .get_result::<User>(self.db_conn)
            .map_err(|e| e.context(format!("Create a new user {:?} error occured", payload)).into())
    }

    /// Updates specific user
    fn update(&self, user_id_arg: UserId, payload: UpdateUser) -> RepoResult<User> {
        let query = users.find(user_id_arg.clone());

        query
            .get_result(self.db_conn)
            .map_err(From::from)
            .and_then(|user: User| acl::check(&*self.acl, Resource::Users, Action::Write, self, Some(&user)))
            .and_then(|_| {
                let filter = users.filter(id.eq(user_id_arg.clone())).filter(is_active.eq(true));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<User>(self.db_conn).map_err(From::from)
            })
            .map_err(|e: FailureError| {
                e.context(format!("update user {} with {:?} error occured", user_id_arg, payload))
                    .into()
            })
    }

    /// Deactivates specific user
    fn deactivate(&self, user_id_arg: UserId) -> RepoResult<User> {
        let query = users.find(user_id_arg.clone());

        query
            .get_result(self.db_conn)
            .map_err(From::from)
            .and_then(|user: User| acl::check(&*self.acl, Resource::Users, Action::Write, self, Some(&user)))
            .and_then(|_| {
                let filter = users.filter(id.eq(user_id_arg.clone())).filter(is_active.eq(true));
                let query = diesel::update(filter).set(is_active.eq(false));

                query.get_result(self.db_conn).map_err(From::from)
            })
            .map_err(|e: FailureError| e.context(format!("Deactivates user {:?} error occured", user_id_arg)).into())
    }

    /// Deletes specific user by saga id
    fn delete_by_saga_id(&self, saga_id_arg: String) -> RepoResult<User> {
        let filtered = users.filter(saga_id.eq(saga_id_arg.clone()));
        let query = diesel::delete(filtered);
        query.get_result(self.db_conn).map_err(|e| {
            e.context(format!("Delete specific user by saga id {:?} error occured", saga_id_arg))
                .into()
        })
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
