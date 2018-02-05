//! Module containing info about enum Provider and its impls of service traits for converting to string in db
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)] 
pub enum Provider {
   Email,
   UnverifiedEmail,
   Facebook,
   Google
}

mod diesel_impl {
    use diesel::Queryable;
    use diesel::expression::AsExpression;
    use diesel::expression::bound::Bound;
    use diesel::pg::Pg;
    use diesel::row::Row;
    use diesel::serialize::{IsNull, ToSql};
    use diesel::serialize::Output;
    use diesel::deserialize::FromSqlRow;
    use diesel::sql_types::*;
    use std::error::Error;
    use std::io::Write;

    use super::{Provider};

    impl<'a> AsExpression<Nullable<Varchar>> for &'a Provider {
        type Expression = Bound<Nullable<Varchar>, &'a Provider>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl AsExpression<Nullable<Varchar>> for Provider {
        type Expression = Bound<Nullable<Varchar>, Provider>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl ToSql<Nullable<Varchar>, Pg> for Provider {
        fn to_sql<W: Write>(
            &self,
            out: &mut Output<W, Pg>,
        ) -> Result<IsNull, Box<Error + Send + Sync>> {
            match *self {
                Provider::Email => out.write_all(b"Email")?,
                Provider::UnverifiedEmail => out.write_all(b"UnverifiedEmail")?,
                Provider::Facebook => out.write_all(b"Facebook")?,
                Provider::Google => out.write_all(b"Google")?,
            }
            Ok(IsNull::No)
        }
    }

    impl FromSqlRow<Nullable<Varchar>, Pg> for Provider {
        fn build_from_row<T: Row<Pg>>(row: &mut T) -> Result<Self, Box<Error + Send + Sync>> {
            match row.take() {
                Some(b"Email") => Ok(Provider::Email),
                Some(b"UnverifiedEmail") => Ok(Provider::UnverifiedEmail),
                Some(b"Facebook") => Ok(Provider::Facebook),
                Some(b"Google") => Ok(Provider::Google),
                Some(_) => Err("Unrecognized enum variant".into()),
                None => Err("Unrecognized enum variant".into()),
            }
        }
    }

    impl Queryable<Nullable<Varchar>, Pg> for Provider {
        type Row = Self;

        fn build(row: Self::Row) -> Self {
            row
        }
    }

}
