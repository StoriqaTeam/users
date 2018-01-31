use std::time::SystemTime;
use std::str::FromStr;

use validator::Validate;

use models::identity::NewIdentity;
use super::authorization::{Scope, WithScope};

table! {
    use diesel::sql_types::*;
    use models::user::GenderType;
    users (id) {
        id -> Integer,
        email -> Varchar,
        email_verified -> Bool,
        phone -> Nullable<VarChar>,
        phone_verified -> Bool,
        is_active -> Bool ,
        first_name -> Nullable<VarChar>,
        last_name -> Nullable<VarChar>,
        middle_name -> Nullable<VarChar>,
        gender -> GenderType,
        birthdate -> Nullable<Timestamp>, //
        last_login_at -> Timestamp, //
        created_at -> Timestamp, // UTC 0, generated at db level
        updated_at -> Timestamp, // UTC 0, generated at db level
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Clone)]
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
#[derive(Serialize, Deserialize, Insertable, Validate, Clone)]
#[table_name = "users"]
pub struct NewUser {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub phone: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub gender: Gender,
    pub birthdate: Option<SystemTime>,
    pub last_login_at: SystemTime,
}

/// Payload for updating users
#[derive(Serialize, Deserialize, Insertable, AsChangeset)]
#[table_name = "users"]
pub struct UpdateUser {
    pub email: Option<String>,
    pub phone: Option<Option<String>>,
    pub first_name: Option<Option<String>>,
    pub last_name: Option<Option<String>>,
    pub middle_name: Option<Option<String>>,
    pub gender: Option<Gender>,
    pub birthdate: Option<Option<SystemTime>>,
}

impl WithScope for User {
    fn is_in_scope(&self, scope: &Scope, user_id: i32) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => self.id == user_id
        }
    }
}

impl From<NewIdentity> for NewUser {
    fn from(identity: NewIdentity) -> Self {
        NewUser {
            email: identity.email,
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
pub enum Gender {
   Male,
   Female,
   Undefined
}

impl FromStr for Gender {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "male" => Ok(Gender::Male),
            "female" => Ok(Gender::Female),
            _ => Ok(Gender::Undefined),
        }
    }
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

    use super::{Gender, GenderType};

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
