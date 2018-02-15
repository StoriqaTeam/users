use std::convert::From;

use diesel;
use diesel::select;
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::pg::PgConnection;

use models::UserId;
use models::{Identity, Provider};
use models::identity::identity::identities::dsl::*;
use super::error::RepoError;
use super::types::DbConnection;

/// Identities repository, responsible for handling identities
pub struct IdentitiesRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
}

pub trait IdentitiesRepo {
    /// Checks if e-mail is already registered
    fn email_provider_exists(&self, email_arg: String, provider: Provider) -> Result<bool, RepoError>;

    /// Creates new identity
    fn create(
        &self,
        email_arg: String,
        password_arg: Option<String>,
        provider_arg: Provider,
        user_id_arg: UserId,
    ) -> Result<Identity, RepoError>;

    /// Verifies password
    fn verify_password(&self, email_arg: String, password_arg: String) -> Result<bool, RepoError>;

    /// Find specific user by email
    fn find_by_email_provider(&self, email_arg: String, provider_arg: Provider) -> Result<Identity, RepoError>;
}

impl<'a> IdentitiesRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection) -> Self {
        Self { db_conn }
    }

    fn execute_query<T: Send + 'static, U: LoadQuery<PgConnection, T> + Send + 'static>(&self, query: U) -> Result<T, RepoError> {
        let conn = self.db_conn;

        query.get_result::<T>(&*conn).map_err(RepoError::from)
    }
}

impl<'a> IdentitiesRepo for IdentitiesRepoImpl<'a> {
    /// Checks if e-mail is already registered
    fn email_provider_exists(&self, email_arg: String, provider_arg: Provider) -> Result<bool, RepoError> {
        self.execute_query(select(exists(
            identities
                .filter(email.eq(email_arg))
                .filter(provider.eq(provider_arg)),
        )))
    }

    /// Verifies password
    fn verify_password(&self, email_arg: String, password_arg: String) -> Result<bool, RepoError> {
        self.execute_query(select(exists(
            identities
                .filter(email.eq(email_arg))
                .filter(password.eq(password_arg)),
        )))
    }

    /// Creates new user
    // TODO - set e-mail uniqueness in database
    fn create(
        &self,
        email_arg: String,
        password_arg: Option<String>,
        provider_arg: Provider,
        user_id_arg: UserId,
    ) -> Result<Identity, RepoError> {
        let identity_arg = Identity {
            user_id: user_id_arg,
            email: email_arg,
            provider: provider_arg,
            password: password_arg,
        };

        let ident_query = diesel::insert_into(identities).values(&identity_arg);
        ident_query
            .get_result::<Identity>(&**self.db_conn)
            .map_err(RepoError::from)
    }

    /// Find specific user by email
    fn find_by_email_provider(&self, email_arg: String, provider_arg: Provider) -> Result<Identity, RepoError> {
        let query = identities
            .filter(email.eq(email_arg))
            .filter(provider.eq(provider_arg));

        query
            .first::<Identity>(&**self.db_conn)
            .map_err(RepoError::from)
    }
}
