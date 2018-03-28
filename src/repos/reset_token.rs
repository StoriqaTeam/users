use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;

use super::error::RepoError;
use super::types::DbConnection;
use models::ResetToken;
use models::reset_token::reset_tokens::dsl::*;

/// Identities repository, responsible for handling identities
pub struct ResetTokenRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
}

pub trait ResetTokenRepo {
    /// Create token for user
    fn create(&self, reset_token_arg: ResetToken) -> Result<ResetToken, RepoError>;

    /// Find by token
    fn find_by_token(&self, token_arg: String) -> Result<ResetToken, RepoError>;

    /// Find by email
    fn find_by_email(&self, email_arg: String) -> Result<ResetToken, RepoError>;

    /// Delete by token
    fn delete_by_token(&self, token_arg: String) -> Result<ResetToken, RepoError>;

    /// Delete by email
    fn delete_by_email(&self, email_arg: String) -> Result<ResetToken, RepoError>;
}

impl<'a> ResetTokenRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection) -> Self {
        Self { db_conn }
    }
}

impl<'a> ResetTokenRepo for ResetTokenRepoImpl<'a> {
    fn create(&self, reset_token_arg: ResetToken) -> Result<ResetToken, RepoError> {
        let insert_query = diesel::insert_into(reset_tokens).values(&reset_token_arg);

        insert_query.get_result::<ResetToken>(&**self.db_conn).map_err(RepoError::from)
    }

    fn find_by_token(&self, token_arg: String) -> Result<ResetToken, RepoError> {
        let query = reset_tokens.filter(token.eq(token_arg));

        query.first::<ResetToken>(&**self.db_conn).map_err(RepoError::from)
    }

    fn find_by_email(&self, email_arg: String) -> Result<ResetToken, RepoError> {
        let query = reset_tokens.filter(email.eq(email_arg));

        query
            .first::<ResetToken>(&**self.db_conn)
            .map_err(RepoError::from)
    }

    fn delete_by_token(&self, token_arg: String) -> Result<ResetToken, RepoError> {
        let filtered = reset_tokens.filter(token.eq(token_arg));
        let query = diesel::delete(filtered);
        query.get_result(&**self.db_conn).map_err(RepoError::from)
    }

    fn delete_by_email(&self, email_arg: String) -> Result<ResetToken, RepoError> {
        let filtered = reset_tokens.filter(email.eq(email_arg));
        let query = diesel::delete(filtered);
        query.get_result(&**self.db_conn).map_err(RepoError::from)
    }
}
