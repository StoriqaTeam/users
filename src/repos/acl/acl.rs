//! Authorization module contains authorization logic for the repo layer app
use std::collections::HashMap;
use std::rc::Rc;

use models::authorization::*;
use repos::error::RepoError;
use repos::types::DbConnection;
use stq_acl::{Acl, CheckScope};

pub fn check<T>(
    acl: &Acl<Resource, Action, Scope, RepoError, T>,
    resource: &Resource,
    action: &Action,
    scope_checker: &CheckScope<Scope, T>,
    obj: Option<&T>,
) -> Result<(), RepoError> {
    acl.allows(resource, action, scope_checker, obj)
        .and_then(|allowed| {
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
            ],
        );
        hash.insert(
            Role::User,
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

    use models::authorization::*;
    use models::*;
    use repos::*;

    use stq_acl::{RolesCache, WithScope};

    struct CacheRolesMock {}

    impl RolesCache for CacheRolesMock {
        type Error = RepoError;
        type Role = Role;

        fn get(&self, id: i32, _con: Option<&DbConnection>) -> Result<Vec<Self::Role>, Self::Error> {
            match id {
                1 => Ok(vec![Role::Superuser]),
                _ => Ok(vec![Role::User]),
            }
        }

        fn clear(&self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn remove(&self, id: i32) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    const MOCK_USER_ROLE: CacheRolesMock = CacheRolesMock {};

    fn create_store() -> Store {
        Store {
            id: 1,
            user_id: 1,
            name: "name".to_string(),
            is_active: true,
            currency_id: 1,
            short_description: "short description".to_string(),
            long_description: None,
            slug: "myname".to_string(),
            cover: None,
            logo: None,
            phone: "1234567".to_string(),
            email: "example@mail.com".to_string(),
            address: "town city street".to_string(),
            facebook_url: None,
            twitter_url: None,
            instagram_url: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }

    #[test]
    fn test_super_user_for_users() {
        let mut acl = ApplicationAcl::new(MOCK_USER_ROLE, 1);

        let resource = create_store();

        let resources = vec![&resource as &WithScope<Scope>];

        assert_eq!(
            acl.allows(Resource::Products, Action::All, &resources, None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.allows(Resource::Products, Action::Read, &resources, None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.allows(Resource::Products, Action::Create, &resources, None)
                .unwrap(),
            true
        );
    }

    #[test]
    fn test_ordinary_user_for_users() {
        let mut acl = ApplicationAcl::new(MOCK_USER_ROLE, 2);

        let resource = create_store();
        let resources = vec![&resource as &WithScope];

        assert_eq!(
            acl.allows(Resource::Products, Action::All, &resources, None)
                .unwrap(),
            false
        );
        assert_eq!(
            acl.allows(Resource::Products, Action::Read, &resources, None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.allows(Resource::Products, Action::Create, &resources, None)
                .unwrap(),
            false
        );
    }

    #[test]
    fn test_super_user_for_user_roles() {
        let mut group = ApplicationAcl::new(MOCK_USER_ROLE, 1);

        let resource = UserRole {
            id: 1,
            user_id: 1,
            role: Role::User,
        };
        let resources = vec![&resource as &WithScope<Scope>];

        assert_eq!(
            group
                .allows(Resource::UserRoles, Action::All, &resources, None)
                .unwrap(),
            true
        );
        assert_eq!(
            group
                .allows(Resource::UserRoles, Action::Read, &resources, None)
                .unwrap(),
            true
        );
        assert_eq!(
            group
                .allows(Resource::UserRoles, Action::Create, &resources, None)
                .unwrap(),
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
        let resources = vec![&resource as &WithScope<Scope>];

        assert_eq!(
            acl.allows(Resource::UserRoles, Action::All, resources.clone(), None)
                .unwrap(),
            false
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Read, resources.clone(), None)
                .unwrap(),
            false
        );
        assert_eq!(
            acl.allows(Resource::UserRoles, Action::Create, resources.clone(), None)
                .unwrap(),
            false
        );
    }

}
