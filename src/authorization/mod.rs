//! Authorization module contains authorization logic for the whole app

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

#[derive(PartialEq, Eq, Hash)]
pub enum Role {
    Superuser,
    User,
}

pub enum Resource {
    Users,
}

pub enum Scope {
    All,
    Owned,
}

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

macro_rules! permission {
    ($resource: expr) => { Permission { resource: resource, action: Action::All, scope: Scope::All }  };
    ($resource: expr, $action: expr) => { Permission { resource: resource, action: $action, scope: Scope::All }  };
    ($resource: expr, $action: expr, $scope: expr) => { Permission { resource: resource, action: $action, scope: $scope }  };
}

struct Authorization {
    acls: HashMap<Role, Vec<Permission>>
}

impl Authorization {
    pub fn new() -> Self {
        Self { acls: HashMap::new() }
    }

    pub fn can(&self, role: Role, resource: Resource, action: Action) -> bool {
        let empty_vec: Vec<Permission> = Vec::new();
        let acls = self.acls.get(&role).unwrap_or(&empty_vec);
        let found_acls = acls.filter
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
