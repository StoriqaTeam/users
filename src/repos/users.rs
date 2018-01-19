use std::sync::Arc;

use diesel;
use diesel::select;
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::pg::PgConnection;
use futures::future;
use futures_cpupool::CpuPool;

use common::{TheConnection, ThePool};
use payloads::user::{NewUser, UpdateUser};
use models::user::User;
use models::schema::users::dsl::*;
use super::error::Error;
use super::types::RepoFuture;

/// Users repository, responsible for handling users
pub struct UsersRepoImpl {
    // Todo - no need for Arc, since pool is itself an ARC-like structure
    pub r2d2_pool: Arc<ThePool>,
    pub cpu_pool: Arc<CpuPool>,
}

pub trait UsersRepo {
    /// Find specific user by ID
    fn find(&self, user_id: i32) -> RepoFuture<User>;

    /// Checks if e-mail is already registered
    fn email_exists(&self, email_arg: String) -> RepoFuture<bool>;

    /// Returns list of users, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoFuture<Vec<User>>;

    /// Creates new user
    fn create(&self, payload: NewUser) -> RepoFuture<User>;

    /// Updates specific user
    fn update(&self, user_id: i32, payload: UpdateUser) -> RepoFuture<User>;

    /// Deactivates specific user
    fn deactivate(&self, user_id: i32) -> RepoFuture<User>;
}

impl UsersRepoImpl {
    fn get_connection(&self) -> TheConnection {
        match self.r2d2_pool.get() {
            Ok(connection) => connection,
            Err(e) => panic!("Error obtaining connection from pool: {}", e),
        }
    }

    fn execute_query<T: Send + 'static, U: LoadQuery<PgConnection, T> + Send + 'static>(
        &self,
        query: U,
    ) -> RepoFuture<T> {
        let conn = match self.r2d2_pool.get() {
            Ok(connection) => connection,
            Err(_) => {
                return Box::new(future::err(
                    Error::Connection("Cannot connect to users db".to_string()),
                ))
            }
        };

        Box::new(self.cpu_pool.spawn_fn(move || {
            query.get_result::<T>(&*conn).map_err(|e| Error::from(e))
        }))
    }
}

impl UsersRepo for UsersRepoImpl {
    /// Find specific user by ID
    fn find(&self, user_id: i32) -> RepoFuture<User> {
        self.execute_query(users.find(user_id))
    }

    /// Checks if e-mail is already registered
    fn email_exists(&self, email_arg: String) -> RepoFuture<bool> {
        self.execute_query(select(exists(users.filter(email.eq(email_arg)))))
    }

    /// Returns list of users, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoFuture<Vec<User>> {
        let conn = self.get_connection();
        let query = users
            .filter(is_active.eq(true))
            .filter(id.gt(from))
            .order(id)
            .limit(count);

        Box::new(self.cpu_pool.spawn_fn(move || {
            query.get_results(&*conn).map_err(|e| Error::from(e))
        }))
    }

    /// Creates new user
    // TODO - set e-mail uniqueness in database
    fn create(&self, payload: NewUser) -> RepoFuture<User> {
        let conn = self.get_connection();

        Box::new(self.cpu_pool.spawn_fn(move || {
            let query = diesel::insert_into(users).values(&payload);
            query.get_result(&*conn).map_err(|e| Error::from(e))
        }))
    }

    /// Updates specific user
    fn update(&self, user_id: i32, payload: UpdateUser) -> RepoFuture<User> {
        let conn = self.get_connection();
        let filter = users.filter(id.eq(user_id)).filter(is_active.eq(true));

        Box::new(self.cpu_pool.spawn_fn(move || {
            let query = diesel::update(filter).set(email.eq(payload.email));
            query.get_result::<User>(&*conn).map_err(|e| Error::from(e))
        }))
    }

    /// Deactivates specific user
    fn deactivate(&self, user_id: i32) -> RepoFuture<User> {
        let filter = users.filter(id.eq(user_id)).filter(is_active.eq(true));
        let query = diesel::update(filter).set(is_active.eq(false));
        self.execute_query(query)
    }
}
