#![allow(unused)]
include!("tests_setup.rs");

#[test]
fn test_get_user() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_users_service(Some(1), handle);
    let work = service.get(UserId(1));
    let result = core.run(work).unwrap();
    assert_eq!(result.id, UserId(1));
}

#[test]
fn test_current_user() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_users_service(Some(1), handle);
    let work = service.current();
    let result = core.run(work).unwrap();
    assert_eq!(result.email, MOCK_EMAIL.to_string());
}

#[test]
fn test_current_user_without_user_email() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_users_service(None, handle);
    let work = service.current();
    let result = core.run(work);
    assert_eq!(result.is_err(), true);
}

#[test]
fn test_list() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_users_service(Some(1), handle);
    let work = service.list(1, 5);
    let result = core.run(work).unwrap();
    assert_eq!(result.len(), 5);
}

#[test]
fn test_create_allready_existed() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_users_service(Some(1), handle);
    let new_ident = create_new_identity(MOCK_EMAIL.to_string(), MOCK_PASSWORD.to_string(), Provider::Email, MOCK_SAGA_ID.to_string());
    let work = service.create(new_ident, None);
    let result = core.run(work);
    assert_eq!(result.is_err(), true);
}

#[test]
fn test_create_user() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_users_service(Some(1), handle);
    let new_ident = create_new_identity("new_user@mail.com".to_string(), MOCK_PASSWORD.to_string(), Provider::Email, MOCK_SAGA_ID.to_string());
    let work = service.create(new_ident, None);
    let result = core.run(work).unwrap();
    assert_eq!(result.email, "new_user@mail.com".to_string());
}

#[test]
fn test_update() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_users_service(Some(1), handle);
    let new_user = create_update_user(MOCK_EMAIL.to_string());
    let work = service.update(UserId(1), new_user);
    let result = core.run(work).unwrap();
    assert_eq!(result.id, UserId(1));
    assert_eq!(result.email, MOCK_EMAIL.to_string());
}

#[test]
fn test_deactivate() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_users_service(Some(1), handle);
    let work = service.deactivate(UserId(1));
    let result = core.run(work).unwrap();
    assert_eq!(result.id, UserId(1));
    assert_eq!(result.is_active, false);
}
