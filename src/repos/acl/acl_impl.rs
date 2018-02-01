//! Authorization module contains authorization logic for the repo layer app

use std::collections::HashMap;

use repos::user_roles::UserRolesRepo;
use models::authorization::*;
use super::CachedRoles;

use super::acl::*;



// TODO: remove info about deleted user from cache
pub struct AclImpl<U: UserRolesRepo + 'static + Clone> {
    acls: HashMap<Role, Vec<Permission>>,
    cached_roles: CachedRoles<U>
}


impl<U: UserRolesRepo + 'static + Clone> AclImpl<U> {
    pub fn new(cached_roles: CachedRoles<U>) -> Self {
        let mut result = Self { acls: HashMap::new(), cached_roles: cached_roles };
        result.add_permission_to_role(Role::Superuser, permission!(Resource::Users));
        result.add_permission_to_role(Role::Superuser, permission!(Resource::UserRoles));
        result.add_permission_to_role(Role::User, permission!(Resource::Users, Action::Read));
        result.add_permission_to_role(Role::User, permission!(Resource::Users, Action::All, Scope::Owned));
        result.add_permission_to_role(Role::User, permission!(Resource::UserRoles, Action::Read, Scope::Owned));
        result
    }

    pub fn add_permission_to_role(&mut self, role: Role, permission: Permission) {
        let permissions = self.get_permissions_for_role(role);
        permissions.push(permission);
    }

    fn get_permissions_for_role(&mut self, role: Role) -> &mut Vec<Permission> {
        self.acls.entry(role).or_insert(Vec::new())
    }
}

impl<U: UserRolesRepo + 'static + Clone> Acl for AclImpl<U> {
    fn can(&mut self, resource: Resource, action: Action, user_id: i32, resources_with_scope: Vec<&WithScope>) -> bool {
        let empty: Vec<Permission> = Vec::new();
        let roles = self.cached_roles.get(user_id);
        let hashed_acls = &self.acls;
        let acls = roles.into_iter()
            .flat_map(|role| hashed_acls.get(&role).unwrap_or(&empty))
            .filter(|permission|
                (permission.resource == resource) &&
                ((permission.action == action) || (permission.action == Action::All))
            )
            .filter(|permission| resources_with_scope.iter().all(|res| res.is_in_scope(&permission.scope, user_id)));

        acls.count() > 0
    }
}