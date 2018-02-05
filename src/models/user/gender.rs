//! Module containing info about enum Gender and its impls of service traits for converting to string in db
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Gender {
   Male,
   Female,
   Undefined
}

impl FromStr for Gender {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Male" => Ok(Gender::Male),
            "Female" => Ok(Gender::Female),
            _ => Ok(Gender::Undefined),
        }
    }
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

    use super::{Gender};

    impl<'a> AsExpression<Nullable<VarChar>> for &'a Gender {
        type Expression = Bound<Nullable<VarChar>, &'a Gender>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl AsExpression<Nullable<VarChar>> for Gender {
        type Expression = Bound<Nullable<VarChar>, Gender>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl ToSql<Nullable<VarChar>, Pg> for Gender {
        fn to_sql<W: Write>(
            &self,
            out: &mut Output<W, Pg>,
        ) -> Result<IsNull, Box<Error + Send + Sync>> {
            match *self {
                Gender::Male => out.write_all(b"Male")?,
                Gender::Female => out.write_all(b"Female")?,
                Gender::Undefined => out.write_all(b"Undefined")?,
            }
            Ok(IsNull::No)
        }
    }

    impl FromSqlRow<Nullable<VarChar>, Pg> for Gender {
        fn build_from_row<T: Row<Pg>>(row: &mut T) -> Result<Self, Box<Error + Send + Sync>> {
            match row.take() {
                Some(b"Male") => Ok(Gender::Male),
                Some(b"Female") => Ok(Gender::Female),
                Some(b"Undefined") => Ok(Gender::Undefined),
                Some(_) => Err("Unrecognized enum variant".into()),
                None => Ok(Gender::Undefined),
            }
        }
    }

    impl Queryable<Nullable<VarChar>, Pg> for Gender {
        type Row = Self;

        fn build(row: Self::Row) -> Self {
            row
        }
    }
}
