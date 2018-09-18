//! Repos is a module responsible for interacting with access control lists

#[macro_use]
pub mod macros;
pub mod legacy_acl;
pub mod roles_cache;

pub use self::roles_cache::RolesCacheImpl;

use std::collections::HashMap;
use std::rc::Rc;

use errors::Error;
use failure::Error as FailureError;
use failure::Fail;

use stq_types::{UserId, UsersRole};

use super::legacy_acl::{Acl, CheckScope};
use models::authorization::*;

pub fn check<T>(
    acl: &Acl<Resource, Action, Scope, FailureError, T>,
    resource: Resource,
    action: Action,
    scope_checker: &CheckScope<Scope, T>,
    obj: Option<&T>,
) -> Result<(), FailureError> {
    acl.allows(resource, action, scope_checker, obj).and_then(|allowed| {
        if allowed {
            Ok(())
        } else {
            Err(Error::Forbidden
                .context(format!("Denied request to do {:?} on {:?}", action, resource))
                .into())
        }
    })
}

/// ApplicationAcl contains main logic for manipulation with recources
#[derive(Clone)]
pub struct ApplicationAcl {
    acls: Rc<HashMap<UsersRole, Vec<Permission>>>,
    roles: Vec<UsersRole>,
    user_id: UserId,
}

impl ApplicationAcl {
    pub fn new(roles: Vec<UsersRole>, user_id: UserId) -> Self {
        let mut hash = ::std::collections::HashMap::new();
        hash.insert(
            UsersRole::Superuser,
            vec![
                permission!(Resource::Users, Action::Read), 
                permission!(Resource::Users, Action::Create), 
                permission!(Resource::UserRoles)
            ],
        );
        hash.insert(
            UsersRole::User,
            vec![
                permission!(Resource::Users, Action::Read),
                permission!(Resource::Users, Action::All, Scope::Owned),
                permission!(Resource::UserRoles, Action::Read, Scope::Owned),
            ],
        );

        ApplicationAcl {
            acls: Rc::new(hash),
            roles,
            user_id,
        }
    }
}

impl<T> Acl<Resource, Action, Scope, FailureError, T> for ApplicationAcl {
    fn allows(
        &self,
        resource: Resource,
        action: Action,
        scope_checker: &CheckScope<Scope, T>,
        obj: Option<&T>,
    ) -> Result<bool, FailureError> {
        let empty: Vec<Permission> = Vec::new();
        let user_id = &self.user_id;
        let hashed_acls = self.acls.clone();
        let acls = self
            .roles
            .iter()
            .flat_map(|role| hashed_acls.get(role).unwrap_or(&empty))
            .filter(|permission| (permission.resource == resource) && ((permission.action == action) || (permission.action == Action::All)))
            .filter(|permission| scope_checker.is_in_scope(*user_id, &permission.scope, obj));

        Ok(acls.count() > 0)
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use stq_types::{UserId, UsersRole};

    use repos::legacy_acl::{Acl, CheckScope};

    use models::*;
    use repos::*;

    fn create_user() -> User {
        User {
            id: UserId(1),
            email: "example@mail.com".to_string(),
            email_verified: false,
            phone: None,
            phone_verified: false,
            is_active: true,
            first_name: None,
            last_name: None,
            middle_name: None,
            gender: None,
            avatar: None,
            birthdate: None,
            last_login_at: SystemTime::now(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            saga_id: "saga_id".to_string(),
            is_blocked: false,
        }
    }

    #[derive(Default)]
    struct ScopeChecker;

    impl CheckScope<Scope, User> for ScopeChecker {
        fn is_in_scope(&self, user_id: UserId, scope: &Scope, obj: Option<&User>) -> bool {
            match *scope {
                Scope::All => true,
                Scope::Owned => {
                    if let Some(user) = obj {
                        user.id == user_id
                    } else {
                        false
                    }
                }
            }
        }
    }

    impl CheckScope<Scope, UserRole> for ScopeChecker {
        fn is_in_scope(&self, user_id: UserId, scope: &Scope, obj: Option<&UserRole>) -> bool {
            match *scope {
                Scope::All => true,
                Scope::Owned => {
                    if let Some(user_role) = obj {
                        user_role.user_id == user_id
                    } else {
                        false
                    }
                }
            }
        }
    }

    #[test]
    fn test_super_user_for_users() {
        let acl = ApplicationAcl::new(vec![UsersRole::Superuser], UserId(1232));
        let s = ScopeChecker::default();
        let resource = create_user();

        assert_eq!(acl.allows(Resource::Users, Action::All, &s, Some(&resource)).unwrap(), false, "ACL allows all actions on user.");
        assert_eq!(acl.allows(Resource::Users, Action::Read, &s, Some(&resource)).unwrap(), true, "ACL does not allow read action on user.");
        assert_eq!(acl.allows(Resource::Users, Action::Create, &s, Some(&resource)).unwrap(), true, "ACL allows create actions on user.");
    }

    #[test]
    fn test_ordinary_user_for_users() {
        let acl = ApplicationAcl::new(vec![UsersRole::User], UserId(2));
        let s = ScopeChecker::default();
        let resource = create_user();

        assert_eq!(acl.allows(Resource::Users, Action::All, &s, Some(&resource)).unwrap(), false);
        assert_eq!(acl.allows(Resource::Users, Action::Read, &s, Some(&resource)).unwrap(), true);
        assert_eq!(acl.allows(Resource::Users, Action::Create, &s, Some(&resource)).unwrap(), false);
    }

    #[test]
    fn test_super_user_for_user_roles() {
        let acl = ApplicationAcl::new(vec![UsersRole::Superuser], UserId(1232));
        let s = ScopeChecker::default();

        let resource = UserRole {
            id: 1,
            user_id: UserId(1),
            role: UsersRole::User,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        assert_eq!(acl.allows(Resource::UserRoles, Action::All, &s, Some(&resource)).unwrap(), true);
        assert_eq!(acl.allows(Resource::UserRoles, Action::Read, &s, Some(&resource)).unwrap(), true);
        assert_eq!(acl.allows(Resource::UserRoles, Action::Create, &s, Some(&resource)).unwrap(), true);
    }

    #[test]
    fn test_user_for_user_roles() {
        let acl = ApplicationAcl::new(vec![UsersRole::User], UserId(2));
        let s = ScopeChecker::default();

        let resource = UserRole {
            id: 1,
            user_id: UserId(1),
            role: UsersRole::User,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        assert_eq!(acl.allows(Resource::UserRoles, Action::All, &s, Some(&resource)).unwrap(), false);
        assert_eq!(acl.allows(Resource::UserRoles, Action::Read, &s, Some(&resource)).unwrap(), false);
        assert_eq!(acl.allows(Resource::UserRoles, Action::Create, &s, Some(&resource)).unwrap(), false);
    }
}
