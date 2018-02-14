//! Authorization module contains authorization logic for the repo layer app
use std::rc::Rc;
use std::collections::HashMap;

use models::authorization::*;
use super::RolesCache;
use repos::types::{DbConnection, RepoResult};

/// Access control layer for repos. It tells if a user can do a certain action with
/// certain resource. All logic for roles and permissions should be hardcoded into implementation
/// of this trait.
pub trait Acl {
    /// Tells if a user with id `user_id` can do `action` on `resource`.
    /// `resource_with_scope` can tell if this resource is in some scope, which is also a part of `acl` for some
    /// permissions. E.g. You can say that a user can do `Create` (`Action`) on `Store` (`Resource`) only if he's the
    /// `Owner` (`Scope`) of the store.
    fn can(
        &mut self,
        resource: Resource,
        action: Action,
        resources_with_scope: Vec<&WithScope>,
        conn: Option<&DbConnection>,
    ) -> RepoResult<bool>;
}

/// SystemACL allows all manipulation with recources for all
#[derive(Clone)]
pub struct SystemACL {}

#[allow(unused)]
impl Acl for SystemACL {
    fn can(
        &mut self,
        resource: Resource,
        action: Action,
        resources_with_scope: Vec<&WithScope>,
        conn: Option<&DbConnection>,
    ) -> RepoResult<bool> {
        Ok(true)
    }
}

impl SystemACL {
    pub fn new() -> Self {
        Self {}
    }
}

/// UnauthorizedACL denies all manipulation with recources for all. It is used for unauthorized users.
#[derive(Clone)]
pub struct UnauthorizedACL {}

#[allow(unused)]
impl Acl for UnauthorizedACL {
    fn can(
        &mut self,
        resource: Resource,
        action: Action,
        resources_with_scope: Vec<&WithScope>,
        conn: Option<&DbConnection>,
    ) -> RepoResult<bool> {
        Ok(false)
    }
}

impl UnauthorizedACL {
    pub fn new() -> Self {
        UnauthorizedACL {}
    }
}

/// ApplicationAcl contains main logic for manipulation with recources
// TODO: remove info about deleted user from cache
#[derive(Clone)]
pub struct ApplicationAcl<R: RolesCache> {
    acls: Rc<HashMap<Role, Vec<Permission>>>,
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
            Role::User,
            vec![
                permission!(Resource::Users, Action::Read),
                permission!(Resource::Users, Action::All, Scope::Owned),
                permission!(Resource::UserRoles, Action::Read, Scope::Owned),
            ],
        );

        ApplicationAcl {
            acls: Rc::new(hash),
            roles_cache: roles_cache,
            user_id: user_id,
        }
    }
}

impl<R: RolesCache> Acl for ApplicationAcl<R> {
    fn can(
        &mut self,
        resource: Resource,
        action: Action,
        resources_with_scope: Vec<&WithScope>,
        conn: Option<&DbConnection>,
    ) -> RepoResult<bool> {
        let empty: Vec<Permission> = Vec::new();
        let user_id = &self.user_id;
        let hashed_acls = self.acls.clone();
        self.roles_cache.get(*user_id, conn).and_then(|vec| {
            let acls = vec.into_iter()
                .flat_map(|role| hashed_acls.get(&role).unwrap_or(&empty))
                .filter(|permission| {
                    (permission.resource == resource) && ((permission.action == action) || (permission.action == Action::All))
                })
                .filter(|permission| {
                    resources_with_scope
                        .iter()
                        .all(|res| res.is_in_scope(&permission.scope, *user_id, conn))
                });

            Ok(acls.count() > 0)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use models::authorization::*;
    use repos::*;
    use models::*;

    struct CacheRolesMock {}

    impl RolesCache for CacheRolesMock {
        fn get(&mut self, id: i32, _con: Option<&DbConnection>) -> RepoResult<Vec<Role>> {
            match id {
                1 => Ok(vec![Role::Superuser]),
                _ => Ok(vec![Role::User]),
            }
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

        let resources = vec![&resource as &WithScope];

        assert_eq!(
            acl.can(Resource::Products, Action::All, resources.clone(), None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.can(Resource::Products, Action::Read, resources.clone(), None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.can(Resource::Products, Action::Create, resources.clone(), None)
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
            acl.can(Resource::Products, Action::All, resources.clone(), None)
                .unwrap(),
            false
        );
        assert_eq!(
            acl.can(Resource::Products, Action::Read, resources.clone(), None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.can(Resource::Products, Action::Create, resources.clone(), None)
                .unwrap(),
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
            acl.can(Resource::UserRoles, Action::All, resources.clone(), None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.can(Resource::UserRoles, Action::Read, resources.clone(), None)
                .unwrap(),
            true
        );
        assert_eq!(
            acl.can(Resource::UserRoles, Action::Create, resources.clone(), None)
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
        let resources = vec![&resource as &WithScope];

        assert_eq!(
            acl.can(Resource::UserRoles, Action::All, resources.clone(), None)
                .unwrap(),
            false
        );
        assert_eq!(
            acl.can(Resource::UserRoles, Action::Read, resources.clone(), None)
                .unwrap(),
            false
        );
        assert_eq!(
            acl.can(Resource::UserRoles, Action::Create, resources.clone(), None)
                .unwrap(),
            false
        );
    }

}
