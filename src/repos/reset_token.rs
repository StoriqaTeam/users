use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::Connection;
use failure::Fail;

use super::types::RepoResult;
use models::reset_token::reset_tokens::dsl::*;
use models::{ResetToken, TokenType};

/// Identities repository, responsible for handling identities
pub struct ResetTokenRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
}

pub trait ResetTokenRepo {
    /// Create token for user
    fn create(&self, reset_token_arg: ResetToken) -> RepoResult<ResetToken>;

    /// Find by token
    fn find_by_token(&self, token_arg: String, token_type_arg: TokenType) -> RepoResult<ResetToken>;

    /// Find by email
    fn find_by_email(&self, email_arg: String, token_type_arg: TokenType) -> RepoResult<ResetToken>;

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
    fn create(&self, reset_token_arg: ResetToken) -> RepoResult<ResetToken> {
        let insert_query = diesel::insert_into(reset_tokens).values(&reset_token_arg);

        insert_query.get_result::<ResetToken>(self.db_conn).map_err(|e| {
            e.context(format!("Create token for user {:?} error occured", reset_token_arg))
                .into()
        })
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
    fn find_by_email(&self, email_arg: String, token_type_arg: TokenType) -> RepoResult<ResetToken> {
        let query = reset_tokens.filter(email.eq(email_arg.clone()).and(token_type.eq(token_type_arg.clone())));

        query.first::<ResetToken>(self.db_conn).map_err(|e| {
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
