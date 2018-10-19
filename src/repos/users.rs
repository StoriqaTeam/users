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

use stq_types::UserId;

use super::acl;
use super::types::RepoResult;
use models::authorization::*;
use models::{NewUser, UpdateUser, User, UsersSearchTerms};
use repos::legacy_acl::*;
use schema::users::dsl::*;

/// Users repository, responsible for handling users
pub struct UsersRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<Acl<Resource, Action, Scope, FailureError, User>>,
}

pub trait UsersRepo {
    /// Get user count
    fn count(&self, only_active_users: bool) -> RepoResult<i64>;

    /// Find specific user by ID
    fn find(&self, user_id: UserId) -> RepoResult<Option<User>>;

    /// Check that user with specified email already exists
    fn email_exists(&self, email_arg: String) -> RepoResult<bool>;

    /// Find specific user by email
    fn find_by_email(&self, email_arg: String) -> RepoResult<Option<User>>;

    /// Returns list of users, limited by `from` and `count` parameters
    fn list(&self, from: UserId, count: i64) -> RepoResult<Vec<User>>;

    /// Creates new user
    fn create(&self, payload: NewUser) -> RepoResult<User>;

    /// Updates specific user
    fn update(&self, user_id: UserId, payload: UpdateUser) -> RepoResult<User>;

    /// Deactivates specific user
    fn deactivate(&self, user_id: UserId) -> RepoResult<User>;

    /// Set block status of specific user
    fn set_block_status(&self, user_id: UserId, is_blocked_arg: bool) -> RepoResult<User>;

    /// Deletes specific user
    fn delete_by_saga_id(&self, saga_id_arg: String) -> RepoResult<User>;

    /// Search users limited by `from`, `skip` and `count` parameters
    fn search(&self, from: Option<UserId>, skip: i64, count: i64, term: UsersSearchTerms) -> RepoResult<Vec<User>>;

    /// Fuzzy search users by email
    fn fuzzy_search_by_email(&self, email_arg: String) -> RepoResult<Vec<User>>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UsersRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<Acl<Resource, Action, Scope, FailureError, User>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> UsersRepo for UsersRepoImpl<'a, T> {
    /// Get user count
    fn count(&self, only_active_users: bool) -> RepoResult<i64> {
        let mut query = users.filter(id.ne(1)).into_boxed();

        if only_active_users {
            query = query.filter(is_active.eq(true));
        }

        acl::check(&*self.acl, Resource::Users, Action::Read, self, None)
            .and_then(|_| query.count().get_result(self.db_conn).map_err(From::from))
            .map_err(|e| FailureError::from(e).context("Count users error occurred").into())
    }

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
            }).map_err(|e: FailureError| e.context(format!("Find specific user {} error occured", user_id_arg)).into())
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
            }).map_err(|e: FailureError| {
                e.context(format!("Find specific user by email {:?} error occured", email_arg))
                    .into()
            })
    }

    /// Returns list of users, limited by `from` and `count` parameters
    fn list(&self, from: UserId, count: i64) -> RepoResult<Vec<User>> {
        let query = users
            .filter(id.ne(1)) // hide user_id == 1
            .filter(is_active.eq(true))
            .filter(id.ge(from))
            .order(id)
            .limit(count);

        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|users_res: Vec<User>| {
                for user in &users_res {
                    acl::check(&*self.acl, Resource::Users, Action::Read, self, Some(&user))?;
                }

                Ok(users_res)
            }).map_err(|e: FailureError| {
                e.context(format!("list of users, limited by {} and {} error occured", from, count))
                    .into()
            })
    }

    /// Creates new user
    fn create(&self, payload: NewUser) -> RepoResult<User> {
        let query_user = diesel::insert_into(users).values(&payload);
        acl::check(&*self.acl, Resource::Users, Action::Create, self, None)?;
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
            .and_then(|user: User| acl::check(&*self.acl, Resource::Users, Action::Update, self, Some(&user)))
            .and_then(|_| {
                let filter = users.filter(id.eq(user_id_arg.clone())).filter(is_active.eq(true));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<User>(self.db_conn).map_err(From::from)
            }).map_err(|e: FailureError| {
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
            .and_then(|user: User| acl::check(&*self.acl, Resource::Users, Action::Delete, self, Some(&user)))
            .and_then(|_| {
                let filter = users.filter(id.eq(user_id_arg.clone())).filter(is_active.eq(true));
                let query = diesel::update(filter).set(is_active.eq(false));

                query.get_result(self.db_conn).map_err(From::from)
            }).map_err(|e: FailureError| e.context(format!("Deactivates user {:?} error occured", user_id_arg)).into())
    }

    /// Set block status of specific user
    fn set_block_status(&self, user_id_arg: UserId, is_blocked_arg: bool) -> RepoResult<User> {
        let query = users.find(user_id_arg.clone());

        query
            .get_result(self.db_conn)
            .map_err(From::from)
            .and_then(|user: User| acl::check(&*self.acl, Resource::Users, Action::Block, self, Some(&user)))
            .and_then(|_| {
                let filter = users.filter(id.eq(user_id_arg.clone()));
                let query = diesel::update(filter).set(is_blocked.eq(is_blocked_arg));

                query.get_result(self.db_conn).map_err(From::from)
            }).map_err(|e: FailureError| {
                e.context(format!("Set Block status for user {:?} error occured", user_id_arg))
                    .into()
            })
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

    /// Search users limited by `from`, `skip` and `count` parameters
    fn search(&self, from: Option<UserId>, skip: i64, count: i64, term: UsersSearchTerms) -> RepoResult<Vec<User>> {
        // hide user_id == 1
        let mut query = users.filter(id.ne(1)).into_boxed();

        if let Some(from_id) = from {
            query = query.filter(id.ge(from_id));
        }
        if skip > 0 {
            query = query.offset(skip);
        }
        if count > 0 {
            query = query.limit(count);
        }

        if let Some(term_email) = term.email {
            query = query.filter(email.like(format!("%{}%", term_email)));
        }
        if let Some(term_phone) = term.phone {
            query = query.filter(phone.eq(term_phone));
        }
        if let Some(term_first_name) = term.first_name {
            query = query.filter(first_name.like(format!("%{}%", term_first_name)));
        }
        if let Some(term_last_name) = term.last_name {
            query = query.filter(last_name.like(format!("%{}%", term_last_name)));
        }
        if let Some(term_is_blocked) = term.is_blocked {
            query = query.filter(is_blocked.eq(term_is_blocked));
        }

        query
            .order(id)
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|users_res: Vec<User>| {
                for user in &users_res {
                    acl::check(&*self.acl, Resource::Users, Action::Read, self, Some(&user))?;
                }

                Ok(users_res)
            }).map_err(|e: FailureError| {
                e.context(format!(
                    "search for users error occured (from id: {:?}, skip: {}, count: {})",
                    from, skip, count
                )).into()
            })
    }

    /// Fuzzy search users by email
    fn fuzzy_search_by_email(&self, term_email: String) -> RepoResult<Vec<User>> {
        let query = users.filter(email.like(format!("%{}%", term_email))).order(id);
        query
            .get_results(self.db_conn)
            .map_err(From::from)
            .and_then(|users_res: Vec<User>| {
                for user in &users_res {
                    acl::check(&*self.acl, Resource::Users, Action::Read, self, Some(&user))?;
                }

                Ok(users_res)
            }).map_err(|e: FailureError| e.context(format!("fuzzy search for users by email error occured")).into())
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, User>
    for UsersRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id_arg: UserId, scope: &Scope, obj: Option<&User>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(user) = obj {
                    user.id == user_id_arg
                } else {
                    false
                }
            }
        }
    }
}
