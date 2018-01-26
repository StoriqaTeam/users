use std::convert::From;

use diesel;
use diesel::select;
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::pg::PgConnection;
use futures::future;
use futures_cpupool::CpuPool;

use models::user::{Identity, Provider};
use models::schema::identities::dsl::*;
use super::error::Error;
use super::types::{DbConnection, DbPool, RepoFuture};

/// Identities repository, responsible for handling identities
#[derive(Clone)]
pub struct IdentitiesRepoImpl {
    // Todo - no need for Arc, since pool is itself an ARC-like structure
    pub r2d2_pool: DbPool,
    pub cpu_pool: CpuPool,
}

pub trait IdentitiesRepo {
    /// Checks if e-mail is already registered
    fn email_provider_exists(&self, email_arg: String, provider: Provider) -> RepoFuture<bool>;

    /// Creates new identity
    fn create(&self, email_arg: String, password_arg: Option<String>, provider_arg: Provider, user_id_arg: i32) -> RepoFuture<Identity>;

    /// Verifies password
    fn verify_password(&self, email_arg: String, password_arg: String) -> RepoFuture<bool>;

    /// Find specific user by email
    fn find_by_email(&self, email_arg: String) -> RepoFuture<Identity>;
}

impl IdentitiesRepoImpl {
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

impl IdentitiesRepo for IdentitiesRepoImpl {
    /// Checks if e-mail is already registered
    fn email_provider_exists(&self, email_arg: String, provider_arg: Provider) -> RepoFuture<bool> {
        self.execute_query(select(exists(
            identities
                .filter(user_email.eq(email_arg))
                .filter(provider.eq(provider_arg)),
        )))
    }

    /// Verifies password
    fn verify_password(&self, email_arg: String, password_arg: String) -> RepoFuture<bool> {
        self.execute_query(select(exists(
            identities
                .filter(user_email.eq(email_arg))
                .filter(user_password.eq(password_arg)),
        )))
    }

    /// Creates new user
    // TODO - set e-mail uniqueness in database
    fn create(&self, email_arg: String, password_arg: Option<String>, provider_arg: Provider, user_id_arg: i32) -> RepoFuture<Identity> {
        let conn = self.get_connection();
        Box::new(self.cpu_pool.spawn_fn(move || {
            let identity_arg = Identity {
                user_id: user_id_arg,
                user_email: email_arg,
                provider: provider_arg,
                user_password: password_arg,
            };
            let ident_query = diesel::insert_into(identities).values(&identity_arg);
            ident_query
                .get_result::<Identity>(&*conn)
                .map_err(Error::from)
        }))
    }

    /// Find specific user by email
    fn find_by_email(&self, email_arg: String) -> RepoFuture<Identity>{
        let conn = self.get_connection();
        let query = identities
            .filter(user_email.eq(email_arg));

        Box::new(self.cpu_pool.spawn_fn(move || {
            query.first::<Identity>(&*conn).map_err(|e| Error::from(e))
        }))
    }
}
