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

/// ApplicationAcl contains main logic for manipulation with resources
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
                permission!(Resource::Users, Action::Block),
                permission!(Resource::Users, Action::Delete),
                permission!(Resource::Users, Action::Update),
                permission!(Resource::UserRoles),
            ],
        );
        hash.insert(
            UsersRole::User,
            vec![
                permission!(Resource::Users, Action::Read, Scope::Owned),
                permission!(Resource::Users, Action::Update, Scope::Owned),
                permission!(Resource::UserRoles, Action::Read, Scope::Owned),
            ],
        );
        hash.insert(
            UsersRole::Moderator,
            vec![
                permission!(Resource::Users, Action::Read),
                permission!(Resource::Users, Action::Block),
                permission!(Resource::UserRoles, Action::Read),
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

        if acls.count() > 0 {
            Ok(true)
        } else {
            error!("Denied request from user {} to do {} on {}.", user_id, action, resource);
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use stq_types::{RoleId, UserId, UsersRole};

    use repos::legacy_acl::{Acl, CheckScope};

    use models::*;
    use repos::*;

    fn create_user(id: UserId) -> User {
        User {
            id,
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
            emarsys_id: None,
            country: None,
            referal: None,
            referer: None,
            utm_marks: None,
            revoke_before: SystemTime::now(),
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
        let resource = create_user(UserId(1));

        assert_eq!(
            acl.allows(Resource::Users, Action::All, &s, Some(&resource)).unwrap(),
            false,
            "ACL allows all actions on user for superuser."
        );
        assert_eq!(
            acl.allows(Resource::Users, Action::Read, &s, Some(&resource)).unwrap(),
            true,
            "ACL does not allow read action on user for superuser."
        );
        assert_eq!(
            acl.allows(Resource::Users, Action::Create, &s, Some(&resource)).unwrap(),
            true,
            "ACL does not allow  create actions on user for superuser."
        );
        assert_eq!(
            acl.allows(Resource::Users, Action::Update, &s, Some(&resource)).unwrap(),
            true,
            "ACL allows update actions on user for superuser."
        );
        assert_eq!(
            acl.allows(Resource::Users, Action::Delete, &s, Some(&resource)).unwrap(),
            true,
            "ACL does not allow  delete actions on user for superuser."
        );
        assert_eq!(
            acl.allows(Resource::Users, Action::Block, &s, Some(&resource)).unwrap(),
            true,
            "ACL does not allow block actions on user for superuser."
        );
    }

    #[test]
    fn test_ordinary_user_for_users() {
        let user_id = UserId(2);
        let acl = ApplicationAcl::new(vec![UsersRole::User], user_id);
        let s = ScopeChecker::default();
        let resource = create_user(user_id);

        assert_eq!(
            acl.allows(Resource::Users, Action::All, &s, Some(&resource)).unwrap(),
            false,
            "ACL allows all actions on user for ordinary_user."
        );
        assert_eq!(
            acl.allows(Resource::Users, Action::Read, &s, Some(&resource)).unwrap(),
            true,
            "ACL does not allow read action on user for ordinary_user."
        );
        assert_eq!(
            acl.allows(Resource::Users, Action::Create, &s, Some(&resource)).unwrap(),
            false,
            "ACL allows create actions on user for ordinary_user."
        );
        assert_eq!(
            acl.allows(Resource::Users, Action::Update, &s, Some(&resource)).unwrap(),
            true,
            "ACL does not allow update actions on user for ordinary_user."
        );
        assert_eq!(
            acl.allows(Resource::Users, Action::Delete, &s, Some(&resource)).unwrap(),
            false,
            "ACL allows delete actions on user for ordinary_user."
        );
        assert_eq!(
            acl.allows(Resource::Users, Action::Block, &s, Some(&resource)).unwrap(),
            false,
            "ACL allows block actions on user for ordinary_user."
        );
    }

    #[test]
    fn test_moderator_for_users() {
        let acl = ApplicationAcl::new(vec![UsersRole::Moderator], UserId(32));
        let s = ScopeChecker::default();
        let resource = create_user(UserId(1));

        assert_eq!(
            acl.allows(Resource::Users, Action::All, &s, Some(&resource)).unwrap(),
            false,
            "ACL allows all actions on user for moderator."
        );
        assert_eq!(
            acl.allows(Resource::Users, Action::Read, &s, Some(&resource)).unwrap(),
            true,
            "ACL does not allow read action on user for moderator."
        );
        assert_eq!(
            acl.allows(Resource::Users, Action::Create, &s, Some(&resource)).unwrap(),
            false,
            "ACL allows create actions on user for moderator."
        );
        assert_eq!(
            acl.allows(Resource::Users, Action::Update, &s, Some(&resource)).unwrap(),
            false,
            "ACL allows update actions on user for moderator."
        );
        assert_eq!(
            acl.allows(Resource::Users, Action::Delete, &s, Some(&resource)).unwrap(),
            false,
            "ACL allows delete actions on user for moderator."
        );
        assert_eq!(
            acl.allows(Resource::Users, Action::Block, &s, Some(&resource)).unwrap(),
            true,
            "ACL does not allow block actions on user for moderator."
        );
    }

    #[test]
    fn test_super_user_for_user_roles() {
        let acl = ApplicationAcl::new(vec![UsersRole::Superuser], UserId(1232));
        let s = ScopeChecker::default();

        assert_eq!(
            acl.allows(Resource::UserRoles, Action::All, &s, None::<&UserRole>).unwrap(),
            true,
            "ACL does not allow all actions on user roles for superuser."
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Read, &s, None::<&UserRole>).unwrap(),
            true,
            "ACL does not allow read action on user roles for superuser."
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Create, &s, None::<&UserRole>).unwrap(),
            true,
            "ACL does not allow  create actions on user roles for superuser."
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Delete, &s, None::<&UserRole>).unwrap(),
            true,
            "ACL does not allow  delete actions on user roles for superuser."
        );
    }

    #[test]
    fn test_ordinary_user_for_user_roles() {
        let user_id = UserId(2);
        let acl = ApplicationAcl::new(vec![UsersRole::User], user_id);
        let s = ScopeChecker::default();
        let resource = UserRole {
            id: RoleId::new(),
            user_id,
            name: UsersRole::User,
            data: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::All, &s, Some(&resource)).unwrap(),
            false,
            "ACL allows all actions on user roles for ordinary_user."
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Read, &s, Some(&resource)).unwrap(),
            true,
            "ACL does not allow read action on user roles for ordinary_user."
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Create, &s, Some(&resource)).unwrap(),
            false,
            "ACL allows create actions on user roles for ordinary_user."
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Delete, &s, Some(&resource)).unwrap(),
            false,
            "ACL allows delete actions on user roles for ordinary_user."
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Read, &s, None::<&UserRole>).unwrap(),
            false,
            "ACL allows read actions on all user roles for ordinary_user."
        );
    }

    #[test]
    fn test_moderator_for_user_roles() {
        let user_id = UserId(2);
        let acl = ApplicationAcl::new(vec![UsersRole::Moderator], user_id);
        let s = ScopeChecker::default();
        let resource = UserRole {
            id: RoleId::new(),
            user_id,
            name: UsersRole::User,
            data: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        assert_eq!(
            acl.allows(Resource::UserRoles, Action::All, &s, Some(&resource)).unwrap(),
            false,
            "ACL allows all actions on user roles for moderator."
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Read, &s, Some(&resource)).unwrap(),
            true,
            "ACL does not allow read action on user roles for moderator."
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Create, &s, Some(&resource)).unwrap(),
            false,
            "ACL allows create actions on user roles for moderator."
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Delete, &s, Some(&resource)).unwrap(),
            false,
            "ACL allows delete actions on user roles for moderator."
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Read, &s, None::<&UserRole>).unwrap(),
            true,
            "ACL does not allow read actions on all user roles for moderator."
        );
    }
}
