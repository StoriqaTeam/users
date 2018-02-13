//! Authorization module contains authorization logic for the repo layer app
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

use models::authorization::*;
use super::RolesCache;

macro_rules! permission {
    ($resource: expr) => { Permission { resource: $resource, action: Action::All, scope: Scope::All }  };
    ($resource: expr, $action: expr) => { Permission { resource: $resource, action: $action, scope: Scope::All }  };
    ($resource: expr, $action: expr, $scope: expr) => { Permission { resource: $resource, action: $action, scope: $scope }  };
}

/// Access control layer for repos. It tells if a user can do a certain action with
/// certain resource. All logic for roles and permissions should be hardcoded into implementation
/// of this trait.
pub trait Acl {
    /// Tells if a user with id `user_id` can do `action` on `resource`.
    /// `resource_with_scope` can tell if this resource is in some scope, which is also a part of `acl` for some
    /// permissions. E.g. You can say that a user can do `Create` (`Action`) on `Store` (`Resource`) only if he's the
    /// `Owner` (`Scope`) of the store.
    fn can(&mut self, resource: Resource, action: Action, resources_with_scope: Vec<&WithScope>) -> bool;
}

/// SystemACL allows all manipulation with recources for all
#[derive(Clone)]
pub struct SystemACL {}

#[allow(unused)]
impl Acl for SystemACL {
    fn can(&mut self, resource: Resource, action: Action, resources_with_scope: Vec<&WithScope>) -> bool {
        true
    }
}

impl SystemACL {
    pub fn new() -> Self {
        Self {}
    }
}

/// UnAuthanticatedACL denies all manipulation with recources for all
#[derive(Clone)]
pub struct UnAuthanticatedACL {}

#[allow(unused)]
impl Acl for UnAuthanticatedACL {
    fn can(&mut self, resource: Resource, action: Action, resources_with_scope: Vec<&WithScope>) -> bool {
        false
    }
}

impl UnAuthanticatedACL {
    pub fn new() -> Self {
        Self {}
    }
}

/// ApplicationAcl contains main logic for manipulation with recources
// TODO: remove info about deleted user from cache
#[derive(Clone)]
pub struct ApplicationAcl<R: RolesCache> {
    acls: Rc<RefCell<HashMap<Role, Vec<Permission>>>>,
    roles_cache: R,
    user_id: i32,
}

impl<R: RolesCache> ApplicationAcl<R> {
    pub fn new(roles_cache: R, user_id: i32) -> Self {
        let mut hash = ::std::collections::HashMap::new();
        hash.insert(
            Role::Superuser,
            vec![
                permission!(Resource::Users),
                permission!(Resource::UserRoles),
            ],
        );
        hash.insert(
            Role::Superuser,
            vec![
                permission!(Resource::Users, Action::Read),
                permission!(Resource::Users, Action::All, Scope::Owned),
                permission!(Resource::UserRoles, Action::Read, Scope::Owned),
            ],
        );

        ApplicationAcl {
            acls: Rc::new(RefCell::new(hash)),
            roles_cache: roles_cache,
            user_id: user_id,
        }
    }
}

impl<R: RolesCache> Acl for ApplicationAcl<R> {
    fn can(&mut self, resource: Resource, action: Action, resources_with_scope: Vec<&WithScope>) -> bool {
        let empty: Vec<Permission> = Vec::new();
        let user_id = &self.user_id;
        let roles = self.roles_cache.get(*user_id);
        let hashed_acls = self.acls.borrow_mut();
        let acls = roles
            .into_iter()
            .flat_map(|role| hashed_acls.get(&role).unwrap_or(&empty))
            .filter(|permission| (permission.resource == resource) && ((permission.action == action) || (permission.action == Action::All)))
            .filter(|permission| {
                resources_with_scope
                    .iter()
                    .all(|res| res.is_in_scope(&permission.scope, *user_id))
            });

        acls.count() > 0
    }
}

#[cfg(test)]
mod tests {

    use hyper::mime;
    use hyper::{Response, StatusCode};
    use hyper::header::{ContentLength, ContentType};
    use tokio_core::reactor::Core;
    use serde_json;

    use models::identity::NewIdentity;
    use controller::utils::{parse_body, read_body};

    struct Cache_Roles_Mock {}

    impl RolesCache for Cache_Roles_Mock {
        fn get(&mut self, id: i32) -> Vec<Role> {
            match id {
                1 => vec![Role::Superuser],
                _ => vec![Role::User],
            }
        }
    }

    const MOCK_USER_ROLE: Cache_Roles_Mock = Cache_Roles_Mock {};

    #[test]
    fn test_super_user_for_users() {
        let mut acl = ApplicationAcl::new(MOCK_USER_ROLE, 1);

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

        assert_eq!(
            acl.can(Resource::Users, Action::All, 1, resources.clone()),
            true
        );
        assert_eq!(
            acl.can(Resource::Users, Action::Read, 1, resources.clone()),
            true
        );
        assert_eq!(
            acl.can(Resource::Users, Action::Write, 1, resources.clone()),
            true
        );
    }

    #[test]
    fn test_ordinary_user_for_users() {
        let mut acl = ApplicationAcl::new(MOCK_USER_ROLE, 2);

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

        assert_eq!(
            acl.can(Resource::Users, Action::All, 2, resources.clone()),
            false
        );
        assert_eq!(
            acl.can(Resource::Users, Action::Read, 2, resources.clone()),
            true
        );
        assert_eq!(
            acl.can(Resource::Users, Action::Write, 2, resources.clone()),
            false
        );
    }

    #[test]
    fn test_super_user_for_user_roles() {
        let mut acl = ApplicationAcl::new(MOCK_USER_ROLE, 1);

        let resource = UserRole {
            id: 1,
            user_id: 1,
            role: Role::User,
        };
        let resources = vec![&resource as &WithScope];

        assert_eq!(
            acl.can(Resource::UserRoles, Action::All, 1, resources.clone()),
            true
        );
        assert_eq!(
            acl.can(Resource::UserRoles, Action::Read, 1, resources.clone()),
            true
        );
        assert_eq!(
            acl.can(Resource::UserRoles, Action::Write, 1, resources.clone()),
            true
        );
    }

    #[test]
    fn test_user_for_user_roles() {
        let mut acl = ApplicationAcl::new(MOCK_USER_ROLE, 2);

        let resource = UserRole {
            id: 1,
            user_id: 1,
            role: Role::User,
        };
        let resources = vec![&resource as &WithScope];

        assert_eq!(
            acl.can(Resource::UserRoles, Action::All, 2, resources.clone()),
            false
        );
        assert_eq!(
            acl.can(Resource::UserRoles, Action::Read, 2, resources.clone()),
            false
        );
        assert_eq!(
            acl.can(Resource::UserRoles, Action::Write, 2, resources.clone()),
            false
        );
    }

}
