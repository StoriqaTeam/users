//! Module containing info about enum TokenType and its impls of service traits for converting to string in db
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    EmailVerify,
    PasswordReset,
    Undefined,
}

impl FromStr for TokenType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EmailVerify" => Ok(TokenType::EmailVerify),
            "PasswordReset" => Ok(TokenType::PasswordReset),
            _ => Ok(TokenType::Undefined),
        }
    }
}

mod diesel_impl {
    use diesel::deserialize::FromSqlRow;
    use diesel::expression::bound::Bound;
    use diesel::expression::AsExpression;
    use diesel::pg::Pg;
    use diesel::row::Row;
    use diesel::serialize::Output;
    use diesel::serialize::{IsNull, ToSql};
    use diesel::sql_types::*;
    use diesel::Queryable;
    use std::error::Error;
    use std::io::Write;

    use super::TokenType;

    impl<'a> AsExpression<VarChar> for &'a TokenType {
        type Expression = Bound<VarChar, &'a TokenType>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl AsExpression<VarChar> for TokenType {
        type Expression = Bound<VarChar, TokenType>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl ToSql<VarChar, Pg> for TokenType {
        fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> Result<IsNull, Box<Error + Send + Sync>> {
            match *self {
                TokenType::EmailVerify => out.write_all(b"EmailVerify")?,
                TokenType::PasswordReset => out.write_all(b"PasswordReset")?,
                TokenType::Undefined => out.write_all(b"Undefined")?,
            }
            Ok(IsNull::No)
        }
    }

    impl FromSqlRow<VarChar, Pg> for TokenType {
        fn build_from_row<T: Row<Pg>>(row: &mut T) -> Result<Self, Box<Error + Send + Sync>> {
            match row.take() {
                Some(b"EmailVerify") => Ok(TokenType::EmailVerify),
                Some(b"PasswordReset") => Ok(TokenType::PasswordReset),
                Some(b"Undefined") => Ok(TokenType::Undefined),
                Some(_) => Err("Unrecognized enum variant".into()),
                None => Ok(TokenType::Undefined),
            }
        }
    }

    impl Queryable<VarChar, Pg> for TokenType {
        type Row = Self;

        fn build(row: Self::Row) -> Self {
            row
        }
    }
}
