use std::time::SystemTime;

use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Fail;
use uuid::Uuid;

use stq_static_resources::TokenType;

use super::types::RepoResult;
use models::ResetToken;
use schema::reset_tokens::dsl::*;

/// Identities repository, responsible for handling identities
pub struct ResetTokenRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
}

pub trait ResetTokenRepo {
    /// Create token for user
    fn upsert(&self, email_arg: String, token_type_arg: TokenType, uuid: Option<Uuid>) -> RepoResult<ResetToken>;

    /// Find by token
    fn find_by_token(&self, token_arg: String, token_type_arg: TokenType) -> RepoResult<ResetToken>;

    /// Find by email
    fn find_by_email(&self, email_arg: String, token_type_arg: TokenType) -> RepoResult<Option<ResetToken>>;

    /// Delete by token
    fn delete_by_token(&self, token_arg: String, token_type_arg: TokenType) -> RepoResult<ResetToken>;

    /// Delete by email
    fn delete_by_email(&self, email_arg: String, token_type_arg: TokenType) -> RepoResult<ResetToken>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ResetTokenRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T) -> Self {
        Self { db_conn }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ResetTokenRepo for ResetTokenRepoImpl<'a, T> {
    /// Create token for user
    fn upsert(&self, email_arg: String, token_type_arg: TokenType, uuid_: Option<Uuid>) -> RepoResult<ResetToken> {
        let filtered = reset_tokens
            .filter(email.eq(email_arg.clone()))
            .filter(token_type.eq(token_type_arg.clone()));
        let token_: Option<ResetToken> = filtered
            .clone()
            .get_result(self.db_conn)
            .optional()
            .map_err(|e| e.context(format!("Get by email {} {:?} error occured", email_arg, token_type_arg)))?;

        if token_.is_some() {
            diesel::update(filtered)
                .set(updated_at.eq(SystemTime::now()))
                .get_result(self.db_conn)
                .map_err(|e| e.context(format!("Update token error occured")).into())
        } else {
            let payload = ResetToken::new(email_arg.clone(), token_type_arg, uuid_);
            diesel::insert_into(reset_tokens)
                .values(payload)
                .get_result::<ResetToken>(self.db_conn)
                .map_err(|e| e.context(format!("Create token for user {:?} error occured", email_arg)).into())
        }
    }

    /// Find by token
    fn find_by_token(&self, token_arg: String, token_type_arg: TokenType) -> RepoResult<ResetToken> {
        let query = reset_tokens.filter(token.eq(token_arg.clone()).and(token_type.eq(token_type_arg.clone())));

        query.first::<ResetToken>(self.db_conn).map_err(|e| {
            e.context(format!("Find by token {}  {:?} error occured", token_arg, token_type_arg))
                .into()
        })
    }

    /// Find by email
    fn find_by_email(&self, email_arg: String, token_type_arg: TokenType) -> RepoResult<Option<ResetToken>> {
        let query = reset_tokens.filter(email.eq(email_arg.clone()).and(token_type.eq(token_type_arg.clone())));

        query.get_result(self.db_conn).optional().map_err(|e| {
            e.context(format!("Find token by email {} {:?} error occured", email_arg, token_type_arg))
                .into()
        })
    }

    /// Delete by token
    fn delete_by_token(&self, token_arg: String, token_type_arg: TokenType) -> RepoResult<ResetToken> {
        let filtered = reset_tokens.filter(token.eq(token_arg.clone()).and(token_type.eq(token_type_arg.clone())));
        let query = diesel::delete(filtered);
        query.get_result(self.db_conn).map_err(|e| {
            e.context(format!("Delete by token {} {:?} error occured", token_arg, token_type_arg))
                .into()
        })
    }

    /// Delete by email
    fn delete_by_email(&self, email_arg: String, token_type_arg: TokenType) -> RepoResult<ResetToken> {
        let filtered = reset_tokens.filter(email.eq(email_arg.clone()).and(token_type.eq(token_type_arg.clone())));
        let query = diesel::delete(filtered);
        query.get_result(self.db_conn).map_err(|e| {
            e.context(format!("Delete by email {} {:?} error occured", email_arg, token_type_arg))
                .into()
        })
    }
}
