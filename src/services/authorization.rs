//! Authorization module contains authorization logic for the whole app

use std::iter;
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

    pub fn can(&self, resource: Resource, action: Action, user_id: i32, user_roles: &[UserRole], resource_with_scope: &WithScope) -> bool {
        let empty: Vec<Permission> = Vec::new();
        let acls = user_roles.iter()
            .map(|user_role| user_role.role.clone())
            .flat_map(|role| self.acls.get(&role).unwrap_or(&empty))
            .filter(|permission|
                (permission.resource == resource) &&
                ((permission.action == action) || (permission.action == Action::All))
            )
            .filter(|permission| resource_with_scope.is_in_scope(&permission.scope, user_id));

        acls.count() > 0
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
