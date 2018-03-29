use std::convert::From;

use diesel;
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::select;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use super::error::RepoError;
use models::UserId;
use models::identity::identity::identities::dsl::*;
use models::{Identity, Provider, UpdateIdentity};

/// Identities repository, responsible for handling identities
pub struct IdentitiesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
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
        saga_id: String,
    ) -> Result<Identity, RepoError>;

    /// Verifies password
    fn verify_password(&self, email_arg: String, password_arg: String) -> Result<bool, RepoError>;

    /// Find specific user by email
    fn find_by_email_provider(&self, email_arg: String, provider_arg: Provider) -> Result<Identity, RepoError>;

    /// Update identity
    fn update(&self, ident: Identity, update: UpdateIdentity) -> Result<Identity, RepoError>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> IdentitiesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T) -> Self {
        Self { db_conn }
    }

    fn execute_query<Q: Send + 'static, U: LoadQuery<T, Q> + Send + 'static>(&self, query: U) -> Result<Q, RepoError> {
        let conn = self.db_conn;

        query.get_result::<Q>(conn).map_err(RepoError::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> IdentitiesRepo for IdentitiesRepoImpl<'a, T> {
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
        saga_id_arg: String,
    ) -> Result<Identity, RepoError> {
        let identity_arg = Identity {
            user_id: user_id_arg,
            email: email_arg,
            provider: provider_arg,
            password: password_arg,
            saga_id: saga_id_arg,
        };

        let ident_query = diesel::insert_into(identities).values(&identity_arg);
        ident_query
            .get_result::<Identity>(self.db_conn)
            .map_err(RepoError::from)
    }

    /// Find specific user by email
    fn find_by_email_provider(&self, email_arg: String, provider_arg: Provider) -> Result<Identity, RepoError> {
        let query = identities
            .filter(email.eq(email_arg))
            .filter(provider.eq(provider_arg));

        query
            .first::<Identity>(self.db_conn)
            .map_err(RepoError::from)
    }

    /// Update identity
    fn update(&self, ident: Identity, update: UpdateIdentity) -> Result<Identity, RepoError> {
        let filter = identities
            .filter(email.eq(ident.email))
            .filter(provider.eq(ident.provider));

        let query = diesel::update(filter).set(&update);
        query
            .get_result::<Identity>(self.db_conn)
            .map_err(RepoError::from)
    }
}
