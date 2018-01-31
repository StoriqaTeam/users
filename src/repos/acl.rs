//! Authorization module contains authorization logic for the repo layer app

use std::collections::HashMap;
use futures::Future;
use futures_cpupool::CpuPool;


use ::models::authorization::*;
use repos::error::Error;
use repos::user_roles::{UserRolesRepo, UserRolesRepoImpl};
use repos::types::DbPool;
use repos::singleton_acl::{get_acl, SingletonAcl};


macro_rules! permission {
    ($resource: expr) => { Permission { resource: $resource, action: Action::All, scope: Scope::All }  };
    ($resource: expr, $action: expr) => { Permission { resource: $resource, action: $action, scope: Scope::All }  };
    ($resource: expr, $action: expr, $scope: expr) => { Permission { resource: $resource, action: $action, scope: $scope }  };
}

/// Access control layer for repos. It tells if a user can do a certain action with
/// certain resource. All logic for roles and permissions should be hardcoded into implementation
/// of this trait.
pub trait Acl {
    /// Tells that a user with id `user_id` can do `action` on `resource`.
    /// `resource_with_scope` can tell if this resource is in some scope, which is also a part of `acl` for some
    /// permissions. E.g. You can say that a user can do `Create` (`Action`) on `Store` (`Resource`) only if he's the
    /// `Owner` (`Scope`) of the store.
    fn can (&mut self, resource: Resource, action: Action, user_id: i32, resources_with_scope: Vec<&WithScope>) -> bool;
}

pub trait Cacheable<ID, T> {
    fn get(&mut self, id: ID) -> &mut T ;
}

pub struct CachedRoles<U: UserRolesRepo + 'static + Clone> {
    roles_cache: HashMap<i32, Vec<Role>>,
    users_role_repo: U,
}

impl<U: UserRolesRepo + 'static + Clone> CachedRoles<U> {
    pub fn new (repo: U) -> Self {
        Self {
            roles_cache: HashMap::new(),
            users_role_repo: repo
        }
    }
}

impl<U: UserRolesRepo + 'static + Clone> Cacheable<i32, Vec<Role>> for CachedRoles<U> {
    fn get(&mut self, id: i32) -> &mut Vec<Role> {
        let id_clone = id.clone();
        let repo = self.users_role_repo.clone();
        self.roles_cache.entry(id_clone)
            .or_insert_with(|| {
                repo
                    .list_for_user(id_clone)
                    .wait()
                    .map(|users| users.into_iter().map(|u| u.role).collect())
                    .unwrap_or_default()
            })
    }
}

// TODO: remove info about deleted user from cache
pub struct AclImpl<T: Cacheable<i32, Vec<Role>>> {
    acls: HashMap<Role, Vec<Permission>>,
    cached_roles: T 
}


impl<T: Cacheable<i32, Vec<Role>>> AclImpl<T> {
    pub fn new(cached_roles: T) -> Self {
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

impl<T: Cacheable<i32, Vec<Role>>> Acl for AclImpl<T> {
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


pub trait AclWithContext {
    fn can(&self, resource: Resource, action: Action, resources_with_scope: Vec<&WithScope>) -> Result<bool, Error>;
}

#[derive(Clone)]
pub struct AclWithContextImpl {
    acl: SingletonAcl,
    user_id: Option<i32>
}

impl AclWithContextImpl {
    pub fn new(r2d2_pool: DbPool, cpu_pool:CpuPool, user_id: Option<i32>) -> Self {
        let user_roles_repo = UserRolesRepoImpl::new(r2d2_pool, cpu_pool);
        let acl = get_acl(user_roles_repo);
        Self {
            acl: acl,
            user_id: user_id
        }
    }
}

impl AclWithContext for AclWithContextImpl {
    fn can(&self, resource: Resource, action: Action, resources_with_scope: Vec<&WithScope>) -> Result<bool, Error> {
        if let Some(id) = self.user_id {
            Ok(self.acl.inner.lock().unwrap().can(resource, action, id, resources_with_scope))
        } else {
            Err(Error::ContstaintViolation(format!("Unauthorized request")))
        }
    }
}




#[cfg(test)]
mod tests {
    use std::time::SystemTime;
    use super::*;
    use ::models::user::*;
    use ::models::user_role::*;

    struct MockCachedRoles {
        roles_cache: HashMap<i32, Vec<Role>>
    }

    impl MockCachedRoles {
        fn new() -> Self {
            let mut hash = HashMap::new();
            hash.insert(1, vec![Role::Superuser]);
            hash.insert(2, vec![Role::User]);
            Self {
                roles_cache: hash
            }
        }
    }


    impl Cacheable<i32, Vec<Role>> for MockCachedRoles {
        fn get(&mut self, id: i32) -> &mut Vec<Role> {
            self.roles_cache.entry(id)
                .or_insert(vec![Role::User])
        }
    }
    

    #[test]
    fn test_super_user_for_users() {
        let cache = MockCachedRoles::new();
        let mut acl = AclImpl::new(cache);

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

        let resources = vec![&resource as &WithScope];

        assert_eq!(acl.can(Resource::Users, Action::All,   1, resources.clone()), true);
        assert_eq!(acl.can(Resource::Users, Action::Read,  1, resources.clone()), true);
        assert_eq!(acl.can(Resource::Users, Action::Write, 1, resources.clone()), true);
    }

    #[test]
    fn test_ordinary_user_for_users() {
        let cache = MockCachedRoles::new();
        let mut acl = AclImpl::new(cache);

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
        let resources = vec![&resource as &WithScope];

        assert_eq!(acl.can(Resource::Users, Action::All,   2, resources.clone()), false);
        assert_eq!(acl.can(Resource::Users, Action::Read,  2, resources.clone()), true);
        assert_eq!(acl.can(Resource::Users, Action::Write, 2, resources.clone()), false);
    }

    #[test]
    fn test_super_user_for_user_roles() {
        let cache = MockCachedRoles::new();
        let mut acl = AclImpl::new(cache);

        let resource = UserRole {
            id: 1,
            user_id: 1,
            role: Role::User,
        };
        let resources = vec![&resource as &WithScope];

        assert_eq!(acl.can(Resource::UserRoles, Action::All,   1, resources.clone()), true);
        assert_eq!(acl.can(Resource::UserRoles, Action::Read,  1, resources.clone()), true);
        assert_eq!(acl.can(Resource::UserRoles, Action::Write, 1, resources.clone()), true);
    }

    #[test]
    fn test_user_for_user_roles() {
        let cache = MockCachedRoles::new();
        let mut acl = AclImpl::new(cache);

        let resource = UserRole {
            id: 1,
            user_id: 1,
            role: Role::User,
        };
        let resources = vec![&resource as &WithScope];

        assert_eq!(acl.can(Resource::UserRoles, Action::All,   2, resources.clone()), false);
        assert_eq!(acl.can(Resource::UserRoles, Action::Read,  2, resources.clone()), false);
        assert_eq!(acl.can(Resource::UserRoles, Action::Write, 2, resources.clone()), false);
    }
}

