extern crate rocket;

use rocket_contrib::{Json, Value};

#[get("/healthcheck")]
fn healthcheck() -> Json<Value> {
    Json(json!({
        "status": "running"
    }))
}

#[error(400)]
fn bad_request() -> Json<Value> {
    Json(json!({
        "status": "error",
        "reason": "Bad request."
    }))
}

#[error(404)]
fn not_found() -> Json<Value> {
    Json(json!({
        "status": "error",
        "reason": "Resource was not found."
    }))
}
