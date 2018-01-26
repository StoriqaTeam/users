//! Authorization module contains authorization logic for the whole app

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use ::models::user_role::UserRole;
use ::models::authorization::*;

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
        // let acls = user_roles.iter()
        //     .map(|user_role| user_role.role)
        //     .flat_map(|role| self.acls.get(&role).iter().flat_map(|permissions| permissions.iter()))
        //     .filter(|permission| (permission.resource == resource) && (permission.action == action));
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
