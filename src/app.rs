use future;
use hyper;
use hyper::server::{Request, Service};
use types::ServerFuture;

use controller::Controller;
use http::utils::{response_with_json, response_with_error};

struct Application {
    controller: Controller,
}

impl Service for Application {
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = ServerFuture;

    fn call(&self, req: Request) -> ServerFuture {
        info!("{:?}", req);

        Box::new(
            self.controller.call(req).then(|res| match res {
                Ok(data) => future::ok(response_with_json(data)),
                Err(err) => future::ok(response_with_error(err))
            })
        )
    }
}
