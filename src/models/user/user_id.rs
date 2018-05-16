//! Module containing info about UserId and its impls of service traits for converting to integer in db

// UserId type
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserId(pub i32);

impl Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for UserId {
    type Err = (String);

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match i32::from_str(s) {
            Ok(i) => Ok(UserId(i)),
            Err(_) => Err("Cannot parse user id".into()),
        }
    }
}

mod diesel_impl {
    use diesel::expression::bound::Bound;
    use diesel::expression::AsExpression;
    use diesel::pg::Pg;
    use diesel::row::Row;
    use diesel::serialize::Output;
    use diesel::sql_types::*;
    use diesel::types::FromSql;
    use diesel::types::FromSqlRow;
    use diesel::types::{IsNull, ToSql};
    use diesel::Queryable;
    use std::error::Error;
    use std::io::Write;

    use super::UserId;

    impl<'a> AsExpression<Integer> for &'a UserId {
        type Expression = Bound<Integer, &'a UserId>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl AsExpression<Integer> for UserId {
        type Expression = Bound<Integer, UserId>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl ToSql<Integer, Pg> for UserId {
        fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> Result<IsNull, Box<Error + Send + Sync>> {
            let f = self.0;
            <i32 as ToSql<Integer, Pg>>::to_sql(&f, out)
        }
    }

    impl FromSqlRow<Integer, Pg> for UserId {
        fn build_from_row<T: Row<Pg>>(row: &mut T) -> Result<Self, Box<Error + Send + Sync>> {
            match FromSql::<Integer, Pg>::from_sql(row.take()) {
                Ok(i) => Ok(UserId(i)),
                Err(_) => Err("Null id!".into()),
            }
        }
    }

    impl Queryable<Integer, Pg> for UserId {
        type Row = Self;

        fn build(row: Self::Row) -> Self {
            row
        }
    }
}
