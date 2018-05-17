use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;

use super::error::RepoError;
use models::reset_token::reset_token::reset_tokens::dsl::*;
use models::{ResetToken, TokenType};

/// Identities repository, responsible for handling identities
pub struct ResetTokenRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
}

pub trait ResetTokenRepo {
    /// Create token for user
    fn create(&self, reset_token_arg: ResetToken) -> Result<ResetToken, RepoError>;

    /// Find by token
    fn find_by_token(&self, token_arg: String, token_type_arg: TokenType) -> Result<ResetToken, RepoError>;

    /// Find by email
    fn find_by_email(&self, email_arg: String, token_type_arg: TokenType) -> Result<ResetToken, RepoError>;

    /// Delete by token
    fn delete_by_token(&self, token_arg: String, token_type_arg: TokenType) -> Result<ResetToken, RepoError>;

    /// Delete by email
    fn delete_by_email(&self, email_arg: String, token_type_arg: TokenType) -> Result<ResetToken, RepoError>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ResetTokenRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T) -> Self {
        Self { db_conn }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ResetTokenRepo for ResetTokenRepoImpl<'a, T> {
    fn create(&self, reset_token_arg: ResetToken) -> Result<ResetToken, RepoError> {
        let insert_query = diesel::insert_into(reset_tokens).values(&reset_token_arg);

        insert_query
            .get_result::<ResetToken>(self.db_conn)
            .map_err(RepoError::from)
    }

    fn find_by_token(&self, token_arg: String, token_type_arg: TokenType) -> Result<ResetToken, RepoError> {
        let query = reset_tokens.filter(token.eq(token_arg).and(token_type.eq(token_type_arg)));

        query
            .first::<ResetToken>(self.db_conn)
            .map_err(RepoError::from)
    }

    fn find_by_email(&self, email_arg: String, token_type_arg: TokenType) -> Result<ResetToken, RepoError> {
        let query = reset_tokens.filter(email.eq(email_arg).and(token_type.eq(token_type_arg)));

        query
            .first::<ResetToken>(self.db_conn)
            .map_err(RepoError::from)
    }

    fn delete_by_token(&self, token_arg: String, token_type_arg: TokenType) -> Result<ResetToken, RepoError> {
        let filtered = reset_tokens.filter(token.eq(token_arg).and(token_type.eq(token_type_arg)));
        let query = diesel::delete(filtered);
        query.get_result(self.db_conn).map_err(RepoError::from)
    }

    fn delete_by_email(&self, email_arg: String, token_type_arg: TokenType) -> Result<ResetToken, RepoError> {
        let filtered = reset_tokens.filter(email.eq(email_arg).and(token_type.eq(token_type_arg)));
        let query = diesel::delete(filtered);
        query.get_result(self.db_conn).map_err(RepoError::from)
    }
}
