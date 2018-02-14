//! ACL Macroses

/// Macros used adding permissions to user.
#[macro_export]
macro_rules! permission {
    ($resource: expr) => { Permission { resource: $resource, action: Action::All, scope: Scope::All }  };
    ($resource: expr, $action: expr) => { Permission { resource: $resource, action: $action, scope: Scope::All }  };
    ($resource: expr, $action: expr, $scope: expr) => { Permission { resource: $resource, action: $action, scope: $scope }  };
}

/// Macros used for checking acl. Works with vec of resources, one resource and no resources.
#[macro_export]
macro_rules! acl {
    ($resources: ident, $acl: expr, $res: expr, $act: expr, $con: expr) => (
        {
            let acl = &mut $acl;
            acl.can($res, $act, $resources, $con).and_then(|result| {
            if result {
                Ok(())
            } else {
                Err(RepoError::Unauthorized($res, $act))
            }})
        }
    );
    ([$cur_res: ident], $acl: expr,$res: expr, $act: expr, $con: expr) => (
        {
            let resources = vec![(& $cur_res as &WithScope)];
            acl!(resources, $acl, $res, $act, $con )
        }
    );

    ([], $acl: expr, $res: expr, $act: expr, $con: expr) => (
        {
            let resources = vec![];
            acl!(resources, $acl, $res, $act, $con )
        }
    );
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use repos::acl::{Acl, SystemACL};
    use repos::error::Error;
    use models::authorization::*;
    use models::*;

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
    fn test_system_acl_one_prod() {
        let mut acl = SystemACL {};
        let store = create_store();
        let res = acl!([store], acl, Resource::Products, Action::Read, None).unwrap();

        assert_eq!(res, ());
    }

    #[test]
    fn test_system_acl_no_prod() {
        let mut acl = SystemACL {};
        let res = acl!([], acl, Resource::Products, Action::Read, None).unwrap();

        assert_eq!(res, ());
    }

    #[test]
    fn test_system_acl_vec_prod() {
        let mut acl = SystemACL {};
        let store = create_store();
        let resources = vec![&store as &WithScope];
        let res = acl!(resources, acl, Resource::Products, Action::Read, None).unwrap();

        assert_eq!(res, ());
    }
}
