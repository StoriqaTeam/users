#![allow(unused)]
include!("tests_setup.rs");

#[test]
fn test_super_user_for_users() {
    let cache = CachedRoles::new(MOCK_USER_ROLE);
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
    let cache = CachedRoles::new(MOCK_USER_ROLE);
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
    let cache = CachedRoles::new(MOCK_USER_ROLE);
    let mut acl = AclImpl::new(cache);

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
    let cache = CachedRoles::new(MOCK_USER_ROLE);
    let mut acl = AclImpl::new(cache);

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
