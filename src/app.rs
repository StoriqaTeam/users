//! Application module contains the top-level config for the app.

use future;
use futures::future::Future;
use hyper;
use hyper::server::{Request, Response, Service};
use types::ServerFuture;

use serde_json;
use http::client::ErrorMessage;

use hyper::StatusCode;
use hyper::mime;
use hyper::header::{ContentLength, ContentType};

use failure::Fail;

use controller;
use controller::ControllerImpl;

pub struct Application {
    pub controller: Controller,
}

impl Service for Application {
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = ServerFuture;

    fn call(&self, req: Request) -> ServerFuture {
        info!("{:?}", req);

        Box::new(self.controller.call(req).then(|res| match res {
            Ok(data) => future::ok(Self::response_with_json(data)),
            Err(err) => future::ok(Self::response_with_error(err)),
        }))
    }
}

impl Application {
    /// Responds with JSON, logs response body
    fn response_with_json(body: String) -> Response {
        info!("{}", body);

        Self::response_with_body(body)
    }

    /// Responds with JSON error, logs response body
    fn response_with_error(error: controller::error::ControllerError) -> Response {
        if let Some(trace) = error.backtrace() {
            error!("Trace: {}", trace);
        }
        error!("{:?}", error);
        let mes = ErrorMessage {
            code: error.code().as_u16(),
            message: error.message(),
        };
        let mes = serde_json::to_string(&mes).unwrap();
        Self::response_with_body(mes).with_status(error.code())
    }

    fn response_with_body(body: String) -> Response {
        Response::new()
            .with_header(ContentLength(body.len() as u64))
            .with_header(ContentType(mime::APPLICATION_JSON))
            .with_status(StatusCode::Ok)
            .with_body(body)
    }
}
