//! Models for authorization like Role, Resource, etc.

use std::error::Error;

use diesel::pg::Pg;
use diesel::row::Row;
use diesel::expression::bound::Bound;
use diesel::expression::AsExpression;
use diesel::types::{FromSqlRow};
use diesel::deserialize::Queryable;
use diesel::sql_types::VarChar;



#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub enum Role {
    Superuser,
    User,
}

#[derive(PartialEq, Eq)]
pub enum Resource {
    Users,
}

#[derive(PartialEq, Eq)]
pub enum Scope {
    All,
    Owned,
}

#[derive(PartialEq, Eq)]
pub enum Action {
    All,
    Index,
    Read,
    Write,
}

pub struct Permission {
    pub resource: Resource,
    pub action: Action,
    pub scope: Scope,
}

impl FromSqlRow<VarChar, Pg> for Role {
    fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error + Send + Sync>> {
        match &(String::build_from_row(row)?)[..] {
            "Superuser" => Ok(Role::Superuser),
            "User" => Ok(Role::User),
            v => Err(format!("Unknown value {} for Role found", v).into()),
        }
        // unimplemented!()
    }
}

impl Queryable<VarChar, Pg> for Role {
    type Row = Role;
    fn build(row: Self::Row) -> Self {
        row
    }
}


impl AsExpression<VarChar> for Role {
    type Expression = Bound<VarChar, Role>;
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<'a> AsExpression<VarChar> for &'a Role {
    type Expression = Bound<VarChar, &'a Role>;
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}
