//! Authorization module contains authorization logic for the whole app

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::error::Error;
use std::io::Write;

use diesel::pg::{Pg, PgConnection};
use diesel::prelude::*;
use diesel::row::Row;
use diesel::dsl::AsExprOf;
use diesel::types::*;
use diesel::expression::bound::Bound;
use diesel::expression::AsExpression;
use diesel::types::{FromSqlRow, SmallInt};

use models::user_role::UserRole;

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
#[repr(i16)]
pub enum Role {
    Superuser = 0,
    User = 1,
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

struct Permission {
    pub resource: Resource,
    pub action: Action,
    pub scope: Scope,
}

impl FromSqlRow<SmallInt, Pg> for Role {
    fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error + Send + Sync>> {
        // match i16::build_from_row(row)? {
        //     0 => Ok(Role::Superuser),
        //     1 => Ok(Role::User),
        //     v => Err(format!("Unknown value {} for Role found", v).into()),
        // }
        unimplemented!()
    }
}

impl AsExpression<SmallInt> for Role {
    type Expression = Bound<SmallInt, Role>;
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}

impl<'a> AsExpression<SmallInt> for &'a Role {
    type Expression = Bound<SmallInt, &'a Role>;
    fn as_expression(self) -> Self::Expression {
        Bound::new(self)
    }
}


macro_rules! permission {
    ($resource: expr) => { Permission { resource: resource, action: Action::All, scope: Scope::All }  };
    ($resource: expr, $action: expr) => { Permission { resource: resource, action: $action, scope: Scope::All }  };
    ($resource: expr, $action: expr, $scope: expr) => { Permission { resource: resource, action: $action, scope: $scope }  };
}

struct Authorization {
    acls: HashMap<Role, Vec<Permission>>,
}

impl Authorization {
    pub fn new(user_roles: &[UserRole]) -> Self {
        Self { acls: HashMap::new() }
    }

    pub fn can(&self, user_roles: &[UserRole], resource: Resource, action: Action) -> bool {
        let acls = user_roles.iter()
            .map(|user_role| user_role.role)
            .flat_map(|role| self.acls.get(&role).iter().flat_map(|permissions| permissions.iter()))
            .filter(|permission| (permission.resource == resource) && (permission.action == action));
        false
    }

    fn add_permission_to_role(&mut self, role: Role, permission: Permission) {
        let permissions = self.get_permissions_for_role(role);
        permissions.push(permission);
    }

    fn get_permissions_for_role(&mut self, role: Role) -> &mut Vec<Permission> {
        match self.acls.entry(role) {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(Vec::new())
        }
    }
}

// macro_rules! generate_acl {
//     ($($role: $expr => $acls:tt),*) => ()
// }
