extern crate hyper;
extern crate users_lib;
extern crate serde_json;
extern crate tokio_core;

use hyper::mime;
use hyper::{StatusCode, Response};
use hyper::header::{ContentLength, ContentType};
use tokio_core::reactor::Core;

use users_lib::responses::status::StatusMessage;
use users_lib::models::user::NewUser;
use users_lib::controller::utils::{parse_body, read_body};

#[test]
fn test_read_body() {
    let message = StatusMessage::new("OK");
    let message_str = serde_json::to_string(&message).unwrap();
    let res = response_with_body(message_str.clone());
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
    let res = response_with_body(message_str.clone());
    let mut core = Core::new().unwrap();
    let work = parse_body::<NewUser>(res.body());
    let result = core.run(work).unwrap();
    assert_eq!(result.email, message.email);
}

fn response_with_body(body: String) -> Response {
    Response::new()
        .with_header(ContentLength(body.len() as u64))
        .with_header(ContentType(mime::APPLICATION_JSON))
        .with_status(StatusCode::Ok)
        .with_body(body)
}