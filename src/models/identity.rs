use validator::Validate;

table! {
    use diesel::sql_types::*;
    use models::identity::ProviderType;
    identities (user_id) {
        user_id -> Integer,
        user_email -> Varchar,
        user_password -> Nullable<Varchar>,
        provider -> ProviderType,
    }
}

/// Payload for creating identity for users
#[derive(Debug, Serialize, Deserialize, Validate, Insertable, Queryable)]
#[table_name = "identities"]
pub struct Identity
{
    pub user_id: i32,
    #[validate(email(message = "Invalid email format"))]
    pub user_email: String,
    #[validate(length(min = "6", max = "30", message = "Password should be between 6 and 30 symbols"))]
    pub user_password: Option<String>,
    pub provider: Provider
}

/// Payload for creating users
#[derive(Serialize, Deserialize, Validate, Clone)]
pub struct NewIdentity {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = "6", max = "30", message = "Password should be between 6 and 30 symbols"))]
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)] 
pub enum Provider {
   Email,
   UnverifiedEmail,
   Facebook,
   Google
}

#[derive(QueryId)]
pub struct ProviderType;

mod impls_for_insert_and_query {
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

    use super::{Provider, ProviderType};

    impl HasSqlType<ProviderType> for Pg {
        fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
            lookup.lookup_type("provider_type")
        }
    }

    impl NotNull for ProviderType {}
    impl SingleValue for ProviderType {}

    impl<'a> AsExpression<ProviderType> for &'a Provider {
        type Expression = Bound<ProviderType, &'a Provider>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl AsExpression<ProviderType> for Provider {
        type Expression = Bound<ProviderType, Provider>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl ToSql<ProviderType, Pg> for Provider {
        fn to_sql<W: Write>(
            &self,
            out: &mut Output<W, Pg>,
        ) -> Result<IsNull, Box<Error + Send + Sync>> {
            match *self {
                Provider::Email => out.write_all(b"email")?,
                Provider::UnverifiedEmail => out.write_all(b"unverified_email")?,
                Provider::Facebook => out.write_all(b"facebook")?,
                Provider::Google => out.write_all(b"google")?,
            }
            Ok(IsNull::No)
        }
    }

    impl FromSqlRow<ProviderType, Pg> for Provider {
        fn build_from_row<T: Row<Pg>>(row: &mut T) -> Result<Self, Box<Error + Send + Sync>> {
            match row.take() {
                Some(b"email") => Ok(Provider::Email),
                Some(b"unverified_email") => Ok(Provider::UnverifiedEmail),
                Some(b"facebook") => Ok(Provider::Facebook),
                Some(b"google") => Ok(Provider::Google),
                Some(_) => Err("Unrecognized enum variant".into()),
                None => Err("Unexpected null for non-null column".into()),
            }
        }
    }

    impl Queryable<ProviderType, Pg> for Provider {
        type Row = Self;

        fn build(row: Self::Row) -> Self {
            row
        }
    }

}