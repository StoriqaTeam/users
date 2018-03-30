#![allow(unused)]
include!("tests_setup.rs");

#[test]
fn test_jwt_email() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_jwt_service(handle);
    let new_user = create_new_email_identity(MOCK_EMAIL.to_string(), MOCK_PASSWORD.to_string());
    let work = service.create_token_email(new_user);
    let result = core.run(work).unwrap();
    assert_eq!(
        result.token,
        "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxfQ.u29q4XLsMSDxPJngHHQV4THkbx-Tn9g7HjcLPEKMT1U"
    );
}

#[test]
fn test_jwt_email_not_found() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_jwt_service(handle);
    let new_user = create_new_email_identity("not found email".to_string(), MOCK_PASSWORD.to_string());
    let work = service.create_token_email(new_user);
    let result = core.run(work);
    assert_eq!(result.is_err(), true);
}

#[test]
fn test_jwt_password_incorrect() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_jwt_service(handle);
    let new_user = create_new_email_identity(MOCK_EMAIL.to_string(), "wrong password".to_string());
    let work = service.create_token_email(new_user);
    let result = core.run(work);
    assert_eq!(result.is_err(), true);
}

// this test is ignored because of expired access code from google
#[test]
#[ignore]
fn test_jwt_google() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_jwt_service(handle);
    let oauth = ProviderOauth {
        token: GOOGLE_TOKEN.to_string(),
    };
    let work = service.create_token_google(oauth);
    let result = core.run(work).unwrap();
    assert_eq!(result.token, "token");
}

// this test is ignored because of expired access code from google
#[test]
#[ignore]
fn test_jwt_facebook() {
    let mut core = Core::new().unwrap();
    let handle = Arc::new(core.handle());
    let service = create_jwt_service(handle);
    let oauth = ProviderOauth {
        token: FACEBOOK_TOKEN.to_string(),
    };
    let work = service.create_token_facebook(oauth);
    let result = core.run(work).unwrap();
    assert_eq!(result.token, "token");
}
