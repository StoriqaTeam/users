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
use super::error::Error;
use super::types::DbConnection;

/// Identities repository, responsible for handling identities
pub struct IdentitiesRepoImpl<'a> {
    pub db_conn: &'a DbConnection
}

pub trait IdentitiesRepo {
    /// Checks if e-mail is already registered
    fn email_provider_exists(&self, email_arg: String, provider: Provider) -> Result<bool, Error>;

    /// Creates new identity
    fn create(&self, email_arg: String, password_arg: Option<String>, provider_arg: Provider, user_id_arg: UserId) -> Result<Identity, Error>;

    /// Verifies password
    fn verify_password(&self, email_arg: String, password_arg: String) -> Result<bool, Error>;

    /// Find specific user by email
    fn find_by_email_provider(&self, email_arg: String, provider_arg: Provider) -> Result<Identity, Error>;
}

impl<'a> IdentitiesRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection) -> Self {
        Self {
            db_conn
        }
    }

    fn execute_query<T: Send + 'static, U: LoadQuery<PgConnection, T> + Send + 'static>(
        &self,
        query: U,
    ) -> Result<T, Error> {
        let conn = self.db_conn;

        query.get_result::<T>(&*conn).map_err(|e| Error::from(e))
    }
}

impl<'a> IdentitiesRepo for IdentitiesRepoImpl<'a> {
    /// Checks if e-mail is already registered
    fn email_provider_exists(&self, email_arg: String, provider_arg: Provider) -> Result<bool, Error> {
        self.execute_query(select(exists(
            identities
                .filter(email.eq(email_arg))
                .filter(provider.eq(provider_arg)),
        )))
    }

    /// Verifies password
    fn verify_password(&self, email_arg: String, password_arg: String) -> Result<bool, Error> {
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
        user_id_arg: UserId
    ) -> Result<Identity, Error> {
        let conn = self.db_conn;

        let identity_arg = Identity {
            user_id: user_id_arg,
            email: email_arg,
            provider: provider_arg,
            password: password_arg,
        };
        
        let ident_query = diesel::insert_into(identities).values(&identity_arg);
        ident_query
            .get_result::<Identity>(&**conn)
            .map_err(Error::from)
    }

    /// Find specific user by email
    fn find_by_email_provider(&self, email_arg: String, provider_arg: Provider) -> Result<Identity, Error> {
        let conn = self.db_conn;
        let query = identities
            .filter(email.eq(email_arg))
            .filter(provider.eq(provider_arg));

        query.first::<Identity>(&**conn).map_err(|e| Error::from(e))
    }
}
