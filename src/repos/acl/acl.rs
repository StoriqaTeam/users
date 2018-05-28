//! Authorization module contains authorization logic for the repo layer app
use std::collections::HashMap;
use std::rc::Rc;

use models::authorization::*;
use repos::error::RepoError;
use stq_acl::{Acl, CheckScope};

pub fn check<T>(
    acl: &Acl<Resource, Action, Scope, RepoError, T>,
    resource: &Resource,
    action: &Action,
    scope_checker: &CheckScope<Scope, T>,
    obj: Option<&T>,
) -> Result<(), RepoError> {
    acl.allows(resource, action, scope_checker, obj).and_then(|allowed| {
        if allowed {
            Ok(())
        } else {
            Err(RepoError::Unauthorized(*resource, *action))
        }
    })
}

/// ApplicationAcl contains main logic for manipulation with recources
#[derive(Clone)]
pub struct ApplicationAcl {
    acls: Rc<HashMap<Role, Vec<Permission>>>,
    roles: Vec<Role>,
    user_id: i32,
}

impl ApplicationAcl {
    pub fn new(roles: Vec<Role>, user_id: i32) -> Self {
        let mut hash = ::std::collections::HashMap::new();
        hash.insert(
            Role::Superuser,
            vec![
                permission!(Resource::Users),
                permission!(Resource::UserRoles),
                permission!(Resource::UserDeliveryAddresses),
            ],
        );
        hash.insert(
            Role::User,
            vec![
                permission!(Resource::Users, Action::Read),
                permission!(Resource::Users, Action::All, Scope::Owned),
                permission!(Resource::UserDeliveryAddresses, Action::Read),
                permission!(Resource::UserDeliveryAddresses, Action::All, Scope::Owned),
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

impl<T> Acl<Resource, Action, Scope, RepoError, T> for ApplicationAcl {
    fn allows(
        &self,
        resource: &Resource,
        action: &Action,
        scope_checker: &CheckScope<Scope, T>,
        obj: Option<&T>,
    ) -> Result<bool, RepoError> {
        let empty: Vec<Permission> = Vec::new();
        let user_id = &self.user_id;
        let hashed_acls = self.acls.clone();
        let acls = self.roles
            .iter()
            .flat_map(|role| hashed_acls.get(role).unwrap_or(&empty))
            .filter(|permission| {
                (permission.resource == *resource) && ((permission.action == *action) || (permission.action == Action::All))
            })
            .filter(|permission| scope_checker.is_in_scope(*user_id, &permission.scope, obj));

        Ok(acls.count() > 0)
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use stq_acl::{Acl, CheckScope};

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
            gender: Gender::Male,
            avatar: None,
            birthdate: None,
            last_login_at: SystemTime::now(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            saga_id: "saga_id".to_string(),
        }
    }

    #[derive(Default)]
    struct ScopeChecker;

    impl CheckScope<Scope, User> for ScopeChecker {
        fn is_in_scope(&self, user_id: i32, scope: &Scope, obj: Option<&User>) -> bool {
            match *scope {
                Scope::All => true,
                Scope::Owned => {
                    if let Some(user) = obj {
                        user.id == UserId(user_id)
                    } else {
                        false
                    }
                }
            }
        }
    }

    impl CheckScope<Scope, UserRole> for ScopeChecker {
        fn is_in_scope(&self, user_id: i32, scope: &Scope, obj: Option<&UserRole>) -> bool {
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
        let acl = ApplicationAcl::new(vec![Role::Superuser], 1232);
        let s = ScopeChecker::default();
        let resource = create_user();

        assert_eq!(acl.allows(&Resource::Users, &Action::All, &s, Some(&resource)).unwrap(), true);
        assert_eq!(acl.allows(&Resource::Users, &Action::Read, &s, Some(&resource)).unwrap(), true);
        assert_eq!(acl.allows(&Resource::Users, &Action::Write, &s, Some(&resource)).unwrap(), true);
    }

    #[test]
    fn test_ordinary_user_for_users() {
        let acl = ApplicationAcl::new(vec![Role::User], 2);
        let s = ScopeChecker::default();
        let resource = create_user();

        assert_eq!(acl.allows(&Resource::Users, &Action::All, &s, Some(&resource)).unwrap(), false);
        assert_eq!(acl.allows(&Resource::Users, &Action::Read, &s, Some(&resource)).unwrap(), true);
        assert_eq!(acl.allows(&Resource::Users, &Action::Write, &s, Some(&resource)).unwrap(), false);
    }

    #[test]
    fn test_super_user_for_user_roles() {
        let acl = ApplicationAcl::new(vec![Role::Superuser], 1232);
        let s = ScopeChecker::default();

        let resource = UserRole {
            id: 1,
            user_id: 1,
            role: Role::User,
        };

        assert_eq!(acl.allows(&Resource::UserRoles, &Action::All, &s, Some(&resource)).unwrap(), true);
        assert_eq!(acl.allows(&Resource::UserRoles, &Action::Read, &s, Some(&resource)).unwrap(), true);
        assert_eq!(acl.allows(&Resource::UserRoles, &Action::Write, &s, Some(&resource)).unwrap(), true);
    }

    #[test]
    fn test_user_for_user_roles() {
        let acl = ApplicationAcl::new(vec![Role::User], 2);
        let s = ScopeChecker::default();

        let resource = UserRole {
            id: 1,
            user_id: 1,
            role: Role::User,
        };

        assert_eq!(acl.allows(&Resource::UserRoles, &Action::All, &s, Some(&resource)).unwrap(), false);
        assert_eq!(acl.allows(&Resource::UserRoles, &Action::Read, &s, Some(&resource)).unwrap(), false);
        assert_eq!(
            acl.allows(&Resource::UserRoles, &Action::Write, &s, Some(&resource)).unwrap(),
            false
        );
    }
}
