use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::dsl::exists;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::query_dsl::RunQueryDsl;
use diesel::select;
use diesel::Connection;
use failure::Error as FailureError;
use failure::Fail;

use stq_static_resources::Provider;
use stq_types::UserId;

use super::types::RepoResult;
use models::{Identity, UpdateIdentity};
use schema::identities::dsl::*;

/// Identities repository, responsible for handling identities
pub struct IdentitiesRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
}

pub trait IdentitiesRepo {
    /// Checks if e-mail is already registered
    fn email_exists(&self, email_arg: String) -> RepoResult<bool>;

    fn email_provider_exists(&self, email_arg: String, provider: Provider) -> RepoResult<bool>;

    /// Creates new identity
    fn create(
        &self,
        email_arg: String,
        password_arg: Option<String>,
        provider_arg: Provider,
        user_id_arg: UserId,
        saga_id: String,
    ) -> RepoResult<Identity>;

    /// Verifies password
    fn verify_password(&self, email_arg: String, password_arg: String) -> RepoResult<bool>;

    /// Find specific user by user_id
    fn find_by_id_provider(&self, user_id_arg: UserId, provider_arg: Provider) -> RepoResult<Identity>;

    /// Find specific user by email
    fn find_by_email_provider(&self, email_arg: String, provider_arg: Provider) -> RepoResult<Identity>;

    /// Update identity
    fn update(&self, ident: Identity, update: UpdateIdentity) -> RepoResult<Identity>;

    // Get by user email
    fn get_by_email(&self, email_arg: String) -> RepoResult<Identity>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> IdentitiesRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T) -> Self {
        Self { db_conn }
    }

    fn execute_query<Q: Send + 'static, U: LoadQuery<T, Q> + Send + 'static>(&self, query: U) -> Result<Q, FailureError> {
        let conn = self.db_conn;

        query.get_result::<Q>(conn).map_err(From::from)
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> IdentitiesRepo for IdentitiesRepoImpl<'a, T> {
    /// Checks if e-mail is already registered
    fn email_exists(&self, email_arg: String) -> RepoResult<bool> {
        self.execute_query(select(exists(identities.filter(email.eq(email_arg.clone())))))
            .map_err(|e| {
                e.context(format!("Checks if e-mail {} is already registered error occurred.", email_arg))
                    .into()
            })
    }

    /// Checks if e-mail with provider is already registered
    fn email_provider_exists(&self, email_arg: String, provider_arg: Provider) -> RepoResult<bool> {
        self.execute_query(select(exists(
            identities
                .filter(email.eq(email_arg.clone()))
                .filter(provider.eq(provider_arg.clone())),
        )))
        .map_err(|e| {
            e.context(format!(
                "Checks if e-mail {} with provider {} is already registered error occurred.",
                email_arg, provider_arg
            ))
            .into()
        })
    }

    /// Creates new user
    fn create(
        &self,
        email_arg: String,
        password_arg: Option<String>,
        provider_arg: Provider,
        user_id_arg: UserId,
        saga_id_arg: String,
    ) -> RepoResult<Identity> {
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
            .map_err(|e| e.context(format!("Creates new identity {:?} error occurred.", identity_arg)).into())
    }

    /// Verifies password
    fn verify_password(&self, email_arg: String, password_arg: String) -> RepoResult<bool> {
        self.execute_query(select(exists(
            identities
                .filter(email.eq(email_arg.clone()))
                .filter(password.eq(password_arg.clone())),
        )))
        .map_err(|e| {
            e.context(format!(
                "Verifies password email {} password {} error occurred.",
                email_arg, password_arg
            ))
            .into()
        })
    }

    /// Find specific user by user_id
    fn find_by_id_provider(&self, user_id_arg: UserId, provider_arg: Provider) -> RepoResult<Identity> {
        let query = identities
            .filter(user_id.eq(user_id_arg.clone()))
            .filter(provider.eq(provider_arg.clone()));

        query.first::<Identity>(self.db_conn).map_err(|e| {
            e.context(format!(
                "Find specific user by user_id {} provider {} error occurred.",
                user_id_arg, provider_arg
            ))
            .into()
        })
    }

    /// Find specific user by email
    fn find_by_email_provider(&self, email_arg: String, provider_arg: Provider) -> RepoResult<Identity> {
        let query = identities
            .filter(email.eq(email_arg.clone()))
            .filter(provider.eq(provider_arg.clone()));

        query.first::<Identity>(self.db_conn).map_err(|e| {
            e.context(format!(
                "Find specific user by email {} provider {} error occurred.",
                email_arg, provider_arg
            ))
            .into()
        })
    }

    /// Update identity
    fn update(&self, ident: Identity, update: UpdateIdentity) -> RepoResult<Identity> {
        let filter = identities
            .filter(email.eq(ident.email.clone()))
            .filter(provider.eq(ident.provider.clone()));

        let query = diesel::update(filter).set(&update);
        query.get_result::<Identity>(self.db_conn).map_err(|e| {
            e.context(format!(
                "Update identity {:?} with new identity {:?} error occurred.",
                ident, update
            ))
            .into()
        })
    }

    // Get by user email
    fn get_by_email(&self, email_arg: String) -> RepoResult<Identity> {
        let query = identities.filter(email.eq(&email_arg));

        query.first::<Identity>(self.db_conn).map_err(|e| {
            e.context(format!("Find specific user by email {} error occurred.", email_arg))
                .into()
        })
    }
}
