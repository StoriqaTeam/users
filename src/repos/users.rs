use std::sync::Arc;

use diesel;
use diesel::select;
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::pg::PgConnection;
use futures_cpupool::{CpuFuture, CpuPool};

use common::{TheConnection, ThePool};
use error::Error as ApiError;
use payloads::user::{NewUser, UpdateUser};
use models::user::{User};
use models::schema::users::dsl::*;

/// Users repository, responsible for handling users
pub struct UsersRepo {
    // Todo - no need for Arc, since pool is itself an ARC-like structure
    pub r2d2_pool: Arc<ThePool>,
    pub cpu_pool: Arc<CpuPool>
}

impl UsersRepo {
    /// Find specific user by ID
    pub fn find(&self, user_id: i32) -> CpuFuture<User, ApiError> {
        self.execute_query(users.find(user_id))
    }

    /// Checks if e-mail is already registered
    pub fn email_exists(&self, email_arg: String) -> CpuFuture<bool, ApiError> {
        self.execute_query(select(exists(users.filter(email.eq(email_arg)))))
    }

    /// Returns list of users, limited by `from` and `count` parameters
    pub fn list(&self, from: i32, count: i64) -> CpuFuture<Vec<User>, ApiError> {
        let conn = self.get_connection();
        let query = users.filter(is_active.eq(true)).filter(id.gt(from)).order(id).limit(count);

        self.cpu_pool.spawn_fn(move || {
            query.get_results(&*conn).map_err(|e| ApiError::from(e))
        })
    }

    /// Creates new user
    // TODO - set e-mail uniqueness in database
    pub fn create(&self, payload: NewUser) -> CpuFuture<User, ApiError> {
        // let query = diesel::insert_into(users).values(&payload);
        // self.execute_query(query)
        self.execute_query_fn(move || {
            let pl = payload;
            diesel::insert_into(users).values(&pl)
        })
        // let conn = self.get_connection();

        // self.cpu_pool.spawn_fn(move || {
        //     let query = diesel::insert_into(users).values(&payload);
        //     query.get_result(&*conn).map_err(|e| ApiError::from(e))
        // })
    }

    /// Updates specific user
    pub fn update(&self, user_id: i32, payload: UpdateUser) -> CpuFuture<User, ApiError> {
        let conn = self.get_connection();
        let filter = users.filter(id.eq(user_id)).filter(is_active.eq(true));

        self.cpu_pool.spawn_fn(move || {
            let query = diesel::update(filter).set(email.eq(payload.email));
            query.get_result::<User>(&*conn).map_err(|e| ApiError::from(e))
        })
    }

    /// Deactivates specific user
    pub fn deactivate(&self, user_id: i32) -> CpuFuture<User, ApiError> {
        let conn = self.get_connection();
        let filter = users.filter(id.eq(user_id)).filter(is_active.eq(true));
        let query = diesel::update(filter).set(is_active.eq(false));

        self.cpu_pool.spawn_fn(move || {
            query.get_result(&*conn).map_err(|e| ApiError::from(e))
        })
    }

    fn get_connection(&self) -> TheConnection {
        match self.r2d2_pool.get() {
            Ok(connection) => connection,
            Err(e) => panic!("Error obtaining connection from pool: {}", e)
        }
    }

    fn execute_query<T: Send + 'static, U: LoadQuery<PgConnection, T> + Send + 'static>(&self, query: U) -> CpuFuture<T, ApiError> {
        let conn = self.get_connection();

        self.cpu_pool.spawn_fn(move || {
            query.get_result::<T>(&*conn).map_err(|e| ApiError::from(e))
        })        
    }

    fn execute_query_fn<T: Send + 'static, U: LoadQuery<PgConnection, T>, V: FnOnce() -> U + Send + 'static>(&self, query_fn: V) -> CpuFuture<T, ApiError> {
        let conn = self.get_connection();

        self.cpu_pool.spawn_fn(move || {
            query_fn().get_result::<T>(&*conn).map_err(|e| ApiError::from(e))
        })        
    }

}
