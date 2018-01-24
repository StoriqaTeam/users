use std::time::SystemTime;

use validator::Validate;
use models::schema::users;
use models::schema::identities;


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


#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct User {
   pub id: i32,
   pub email: String,
   pub email_verified: bool,
   pub phone: Option<String>,
   pub phone_verified: bool,
   pub is_active: bool ,
   pub first_name: Option<String>,
   pub last_name: Option<String>,
   pub middle_name: Option<String>,
   pub gender: Gender,
   pub birthdate: Option<SystemTime>,
   pub last_login_at: SystemTime, 
   pub created_at: SystemTime, 
   pub updated_at: SystemTime, 
}

/// Payload for creating users
#[derive(Serialize, Deserialize, Validate, Clone)]
pub struct NewUser {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = "6", max = "30", message = "Password should be between 6 and 30 symbols"))]
    pub password: String,
}

/// Payload for updating users
#[derive(Serialize, Deserialize, Insertable, Validate)]
#[table_name = "users"]
pub struct UpdateUser {
    #[validate(email(message = "Invalid e-mail format"))]
    pub email: String,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub gender: Gender,
    pub birthdate: Option<SystemTime>,
    pub last_login_at: SystemTime, 
}

impl From<NewUser> for UpdateUser {
    fn from(new_user: NewUser) -> Self {
        UpdateUser {
            email: new_user.email,
            phone: None,
            first_name: None,
            last_name: None,
            middle_name:  None,
            gender: Gender::Undefined,
            birthdate: None,
            last_login_at: SystemTime::now(),
        }
    }
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)] 
pub enum Gender {
   Male,
   Female,
   Undefined
}

#[derive(QueryId)]
pub struct GenderType;

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
    use super::{Gender, GenderType};

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


    impl HasSqlType<GenderType> for Pg {
        fn metadata(lookup: &Self::MetadataLookup) -> Self::TypeMetadata {
            lookup.lookup_type("gender_type")
        }
    }

    impl NotNull for GenderType {}
    impl SingleValue for GenderType {}

    impl<'a> AsExpression<GenderType> for &'a Gender {
        type Expression = Bound<GenderType, &'a Gender>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl AsExpression<GenderType> for Gender {
        type Expression = Bound<GenderType, Gender>;

        fn as_expression(self) -> Self::Expression {
            Bound::new(self)
        }
    }

    impl ToSql<GenderType, Pg> for Gender {
        fn to_sql<W: Write>(
            &self,
            out: &mut Output<W, Pg>,
        ) -> Result<IsNull, Box<Error + Send + Sync>> {
            match *self {
                Gender::Male => out.write_all(b"male")?,
                Gender::Female => out.write_all(b"female")?,
                Gender::Undefined => out.write_all(b"undefined")?,
            }
            Ok(IsNull::No)
        }
    }

    impl FromSqlRow<GenderType, Pg> for Gender {
        fn build_from_row<T: Row<Pg>>(row: &mut T) -> Result<Self, Box<Error + Send + Sync>> {
            match row.take() {
                Some(b"male") => Ok(Gender::Male),
                Some(b"female") => Ok(Gender::Female),
                Some(b"undefined") => Ok(Gender::Undefined),
                Some(_) => Err("Unrecognized enum variant".into()),
                None => Err("Unexpected null for non-null column".into()),
            }
        }
    }

    impl Queryable<GenderType, Pg> for Gender {
        type Row = Self;

        fn build(row: Self::Row) -> Self {
            row
        }
    }
}