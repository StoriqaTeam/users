//! Authorization module contains authorization logic for the whole app

use std::iter;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::time::SystemTime;

use ::models::user_role::UserRole;
use ::models::authorization::*;
use ::models::user::Gender;

macro_rules! permission {
    ($resource: expr) => { Permission { resource: $resource, action: Action::All, scope: Scope::All }  };
    ($resource: expr, $action: expr) => { Permission { resource: $resource, action: $action, scope: Scope::All }  };
    ($resource: expr, $action: expr, $scope: expr) => { Permission { resource: $resource, action: $action, scope: $scope }  };
}

struct Acl {
    acls: HashMap<Role, Vec<Permission>>,
}

impl Acl {
    pub fn new() -> Self {
        let mut result = Self { acls: HashMap::new() };
        result.add_permission_to_role(Role::Superuser, permission!(Resource::Users));
        result.add_permission_to_role(Role::User, permission!(Resource::Users, Action::Read));
        result.add_permission_to_role(Role::User, permission!(Resource::Users, Action::All, Scope::Owned));
        result
    }

    pub fn can(&self, resource: Resource, action: Action, roles: &[Role], user_id: i32,resource_with_scope: &WithScope) -> bool {
        let empty: Vec<Permission> = Vec::new();
        let acls = roles.iter()
            .flat_map(|role| self.acls.get(&role).unwrap_or(&empty))
            .filter(|permission|
                (permission.resource == resource) &&
                ((permission.action == action) || (permission.action == Action::All))
            )
            .filter(|permission| resource_with_scope.is_in_scope(&permission.scope, user_id));

        acls.count() > 0
    }

    pub fn add_permission_to_role(&mut self, role: Role, permission: Permission) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ::models::user::*;
    use ::models::authorization::*;

    #[test]
    fn test_super_user() {
        let acl = Acl::new();

        let resource = User {
            id: 1,
            email: "karasev.alexey@gmail.com".to_string(),
            email_verified: true,
            phone: None,
            phone_verified: true,
            is_active: false,
            first_name: None,
            last_name: None,
            middle_name: None,
            gender: Gender::Undefined,
            birthdate: None,
            last_login_at: SystemTime::now(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        assert_eq!(acl.can(Resource::Users, Action::All, &vec![Role::Superuser][..], 1, &resource), true);
        assert_eq!(acl.can(Resource::Users, Action::All, &vec![Role::Superuser][..], 2, &resource), true);
        assert_eq!(acl.can(Resource::Users, Action::Read, &vec![Role::Superuser][..], 1, &resource), true);
        assert_eq!(acl.can(Resource::Users, Action::Read, &vec![Role::Superuser][..], 2, &resource), true);
        assert_eq!(acl.can(Resource::Users, Action::Write, &vec![Role::Superuser][..], 1, &resource), true);
        assert_eq!(acl.can(Resource::Users, Action::Write, &vec![Role::Superuser][..], 2, &resource), true);
        assert_eq!(acl.can(Resource::Users, Action::Index, &vec![Role::Superuser][..], 1, &resource), true);
        assert_eq!(acl.can(Resource::Users, Action::Index, &vec![Role::Superuser][..], 2, &resource), true);
    }

        #[test]
    fn test_ordinary_user() {
        let acl = Acl::new();

        let resource = User {
            id: 1,
            email: "karasev.alexey@gmail.com".to_string(),
            email_verified: true,
            phone: None,
            phone_verified: true,
            is_active: false,
            first_name: None,
            last_name: None,
            middle_name: None,
            gender: Gender::Undefined,
            birthdate: None,
            last_login_at: SystemTime::now(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        assert_eq!(acl.can(Resource::Users, Action::All, &vec![Role::User][..], 1, &resource), true);
        assert_eq!(acl.can(Resource::Users, Action::All, &vec![Role::User][..], 2, &resource), false);
        assert_eq!(acl.can(Resource::Users, Action::Read, &vec![Role::User][..], 1, &resource), true);
        assert_eq!(acl.can(Resource::Users, Action::Read, &vec![Role::User][..], 2, &resource), true);
        assert_eq!(acl.can(Resource::Users, Action::Write, &vec![Role::User][..], 1, &resource), true);
        assert_eq!(acl.can(Resource::Users, Action::Write, &vec![Role::User][..], 2, &resource), false);
        assert_eq!(acl.can(Resource::Users, Action::Index, &vec![Role::User][..], 1, &resource), true);
        assert_eq!(acl.can(Resource::Users, Action::Index, &vec![Role::User][..], 2, &resource), false);
    }

}

