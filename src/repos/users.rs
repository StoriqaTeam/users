use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::pg::PgConnection;
use futures::future;
use futures_cpupool::CpuPool;

use models::user::{UpdateUser, User};
use models::schema::users::dsl::*;
use super::error::Error;
use super::types::{DbConnection, DbPool, RepoFuture};

/// Users repository, responsible for handling users
#[derive(Clone)]
pub struct UsersRepoImpl {
    // Todo - no need for Arc, since pool is itself an ARC-like structure
    pub r2d2_pool: DbPool,
    pub cpu_pool: CpuPool
}

pub trait UsersRepo {
    /// Find specific user by ID
    fn find(&self, user_id: i32) -> RepoFuture<User>;

    /// Find specific user by email
    fn find_by_email(&self, email_arg: String) -> RepoFuture<User>;

    /// Returns list of users, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoFuture<Vec<User>>;

    /// Creates new user
    fn create(&self, payload: UpdateUser) -> RepoFuture<User>;

    /// Updates specific user
    fn update(&self, user_id: i32, payload: UpdateUser) -> RepoFuture<User>;

    /// Deactivates specific user
    fn deactivate(&self, user_id: i32) -> RepoFuture<User>;
}

impl UsersRepoImpl {
    pub fn new(r2d2_pool: DbPool, cpu_pool: CpuPool) -> Self {
        Self {
            r2d2_pool,
            cpu_pool
        }
    }

    fn get_connection(&self) -> DbConnection {
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
    fn find(&self, user_id_arg: i32) -> RepoFuture<User> {
        self.execute_query(users.find(user_id_arg))
    }

    /// Find specific user by email
    fn find_by_email(&self, email_arg: String) -> RepoFuture<User>{
        let conn = self.get_connection();
        let query = users
            .filter(email.eq(email_arg));

        Box::new(self.cpu_pool.spawn_fn(move || {
            query.first::<User>(&*conn).map_err(|e| Error::from(e))
        }))
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
    fn create(&self, payload: UpdateUser) -> RepoFuture<User> {
        let conn = self.get_connection();

        Box::new(self.cpu_pool.spawn_fn(move || {
            let query_user = diesel::insert_into(users).values(&payload);
            query_user
                .get_result::<User>(&*conn)
                .map_err(Error::from)
        }))
    }

    /// Updates specific user
    fn update(&self, user_id_arg: i32, payload: UpdateUser) -> RepoFuture<User> {
        let conn = self.get_connection();
        let filter = users.filter(id.eq(user_id_arg)).filter(is_active.eq(true));

        Box::new(self.cpu_pool.spawn_fn(move || {
            let query = diesel::update(filter).set(email.eq(payload.email));
            query.get_result::<User>(&*conn).map_err(|e| Error::from(e))
        }))
    }

    /// Deactivates specific user
    fn deactivate(&self, user_id_arg: i32) -> RepoFuture<User> {
        let filter = users.filter(id.eq(user_id_arg)).filter(is_active.eq(true));
        let query = diesel::update(filter).set(is_active.eq(false));
        self.execute_query(query)
    }
}
