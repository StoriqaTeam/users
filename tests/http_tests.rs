extern crate hyper;
extern crate users_lib;
extern crate serde_json;
extern crate tokio_core;

use hyper::StatusCode;
use tokio_core::reactor::Core;

use users_lib::error::Error;
use users_lib::responses::status::StatusMessage;
use users_lib::payloads::user::NewUser;
use users_lib::utils::http::{parse_body, read_body, response_not_found, response_with_error, response_with_json};

#[test]
fn test_response_not_found() {
    let res = response_not_found();
    assert_eq!(res.status(), StatusCode::NotFound);
}

#[test]
fn test_response_with_error() {
    let res = response_with_error(Error::InternalServerError);
    assert_eq!(res.status(), Error::InternalServerError.to_code());
}

#[test]
fn test_response_with_json() {
    let message = StatusMessage::new("OK");
    let message_str = serde_json::to_string(&message).unwrap();
    let res = response_with_json(message_str);
    assert_eq!(res.status(), StatusCode::Ok);
}

#[test]
fn test_read_body() {
    let message = StatusMessage::new("OK");
    let message_str = serde_json::to_string(&message).unwrap();
    let res = response_with_json(message_str.clone());
    let body = res.body();
    let mut core = Core::new().unwrap();
    let work = read_body(body);
    let result = core.run(work).unwrap();
    assert_eq!(result, message_str);
}

#[test]
fn test_parse_body() {
    let message = NewUser {
        email: "aaa@mail.com".to_string(),
        password: "password".to_string(),
    };
    let message_str = serde_json::to_string(&message).unwrap();
    let res = response_with_json(message_str.clone());
    let mut core = Core::new().unwrap();
    let work = parse_body::<NewUser>(res.body());
    let result = core.run(work).unwrap();
    assert_eq!(result.email, message.email);
}
