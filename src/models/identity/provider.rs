//! Module containing info about enum Provider and its impls of service traits for converting to string in db
use std::fmt;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Provider {
    Email,
    UnverifiedEmail,
    Facebook,
    Google,
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Provider::Email => "Email",
                Provider::UnverifiedEmail => "UnverifiedEmail",
                Provider::Facebook => "Facebook",
                Provider::Google => "Google",
            }
        )
    }
}

impl FromStr for Provider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Email" => Provider::Email,
            "UnverifiedEmail" => Provider::UnverifiedEmail,
            "Facebook" => Provider::Facebook,
            "Google" => Provider::Google,
            _ => {
                return Err("Unrecognized enum variant".into());
            }
        })
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

    use super::FromStr;
    use super::Provider;

    impl<'a> AsExpression<Varchar> for &'a Provider {
        type Expression = Bound<Varchar, &'a Provider>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl AsExpression<Varchar> for Provider {
        type Expression = Bound<Varchar, Provider>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl ToSql<Varchar, Pg> for Provider {
        fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> Result<IsNull, Box<Error + Send + Sync>> {
            out.write_all(self.to_string().as_bytes())?;
            Ok(IsNull::No)
        }
    }

    impl FromSqlRow<Varchar, Pg> for Provider {
        fn build_from_row<T: Row<Pg>>(row: &mut T) -> Result<Self, Box<Error + Send + Sync>> {
            match row.take() {
                Some(v) => Provider::from_str(&String::from_utf8_lossy(v)),
                None => Err("Unrecognized enum variant".into()),
            }.map_err(|s| s.into())
        }
    }

    impl Queryable<Varchar, Pg> for Provider {
        type Row = Self;

        fn build(row: Self::Row) -> Self {
            row
        }
    }

}
