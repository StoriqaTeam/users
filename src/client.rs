use std::mem;

use tokio_core::reactor::Handle;
use hyper;
use futures::future::IntoFuture;
use futures::{future, Future};
use futures::sync::{mpsc, oneshot};
use futures::stream::Stream;
use futures::sink::Sink;
use serde_json;
use serde::de::Deserialize;
use error::Error;
use hyper_tls;
use super::utils::http;

use settings::Settings;
use responses::error::ErrorMessage;

/// Client for https connections
pub type HyperClient = hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>;

/// Result, transmitted in oneshot channel
pub type ClientResult = Result<String, Error>;

/// Http client   
pub struct Client {
    client: HyperClient,
    tx: mpsc::Sender<Payload>,
    rx: mpsc::Receiver<Payload>,
    max_retries: usize,
}

impl Client {
    /// Create new http client with settings and handle reference
    pub fn new(settings: &Settings, handle: &Handle) -> Self {
        let max_retries = settings.client.http_client_retries;
        let dns_worker_thread_count = settings.client.dns_worker_thread_count;

        let (tx, rx) = mpsc::channel::<Payload>(settings.client.http_client_buffer_size);
        let client = hyper::Client::configure()
            .connector(hyper_tls::HttpsConnector::new(dns_worker_thread_count, &handle).unwrap())
            .no_proto()
            .build(&handle);

        Client {
            client,
            tx,
            rx,
            max_retries,
        }
    }
    
    /// Fetches stream from client 
    pub fn stream(self) -> Box<Stream<Item = (), Error = ()>> {
        let Self {
            client,
            tx: _,
            rx,
            max_retries: _,
        } = self;
        Box::new(rx.and_then(move |payload| {
            Self::send_request(&client, payload)
                .map(|_| ())
                .map_err(|_| ())
        }))
    }

    /// Creates ClientHandle using Client channel
    pub fn handle(&self) -> ClientHandle {
        ClientHandle {
            tx: self.tx.clone(),
            max_retries: self.max_retries,
        }
    }

    fn send_request(client: &HyperClient, payload: Payload) -> Box<Future<Item = (), Error = ()>> {
        let Payload {
            url,
            method,
            body: maybe_body,
            headers: maybe_headers,
            callback,
        } = payload;

        let uri = match url.parse() {
            Ok(val) => val,
            Err(err) => {
                error!(
                    "Url `{}` passed to http client cannot be parsed: `{}`",
                    url, err
                );
                return Box::new(
                    callback
                        .send(Err(Error::BadRequest(format!(
                            "Cannot parse url `{}`",
                            url
                        ))))
                        .into_future()
                        .map(|_| ())
                        .map_err(|_| ()),
                );
            }
        };
        let mut req = hyper::Request::new(method, uri);

        if let Some(headers) = maybe_headers {
            mem::replace(req.headers_mut(), headers);
        }
        
        for body in maybe_body.iter() {
            req.set_body(body.clone());
        }

        let task = client
            .request(req)
            .map_err(|err| err.into())
            .and_then(move |res| {
                let status = res.status();
                let body_future: Box<future::Future<Item = String, Error = Error>> =
                    Box::new(http::read_body(res.body()).map_err(|err| err.into()));
                match status {
                    hyper::StatusCode::Ok => body_future,

                    _ => Box::new(body_future.and_then(move |body| {
                        let message = serde_json::from_str::<ErrorMessage>(&body).ok();
                        let text = match message {
                            Some(m) => m.message,
                            None => "unknown error".to_owned(),
                        };
                        let error = Error::BadRequest(text);
                        future::err(error)
                    })),
                }
            })
            .then(|result| callback.send(result))
            .map(|_| ())
            .map_err(|_| ());

        Box::new(task)
    }
}

/// Client handle for sending data to http client
#[derive(Clone)]
pub struct ClientHandle {
    tx: mpsc::Sender<Payload>,
    max_retries: usize,
}

impl ClientHandle {
    /// Sends http request with use of Method, url, body, headers   
    pub fn request<T>(
        &self,
        method: hyper::Method,
        url: String,
        body: Option<String>,
        headers: Option<hyper::Headers>,
    ) -> Box<Future<Item = T, Error = Error>>
    where
        T: for<'a> Deserialize<'a> + 'static,
    {
        Box::new(
            self.send_request_with_retries(method, url, body, headers, None, self.max_retries)
                .and_then(|response| {
                    serde_json::from_str::<T>(&response)
                        .map_err(|err| Error::BadRequest(format!("{}", err)))
                }),
        )
    }

    fn send_request_with_retries(
        &self,
        method: hyper::Method,
        url: String,
        body: Option<String>,
        headers: Option<hyper::Headers>,
        last_err: Option<Error>,
        retries: usize,
    ) -> Box<Future<Item = String, Error = Error>> {
        if retries == 0 {
            let error = last_err.unwrap_or(Error::BadRequest(
                "Unexpected missing error in send_request_with_retries".to_string(),
            ));
            Box::new(future::err(error))
        } else {
            let self_clone = self.clone();
            let method_clone = method.clone();
            let body_clone = body.clone();
            let headers_clone = headers.clone();
            let url_clone = url.clone();
            Box::new(
                self.send_request(method, url, body, headers)
                    .or_else(move |err| match err {
                        Error::BadRequest(err) => {
                            warn!(
                                "Failed to fetch `{}` with error `{}`, retrying... Retries left {}",
                                url_clone, err, retries
                            );
                            self_clone.send_request_with_retries(
                                method_clone,
                                url_clone,
                                body_clone,
                                headers_clone,
                                Some(Error::BadRequest(err)),
                                retries - 1,
                            )
                        }
                        _ => Box::new(future::err(err)),
                    }),
            )
        }
    }

    fn send_request(
        &self,
        method: hyper::Method,
        url: String,
        body: Option<String>,
        headers: Option<hyper::Headers>,
    ) -> Box<Future<Item = String, Error = Error>> {
        info!(
            "Starting outbound http request: {} {} with body {} with headers {}",
            method,
            url,
            body.clone().unwrap_or_default(),
            headers.clone().unwrap_or_default(),
        );
        let url_clone = url.clone();
        let method_clone = method.clone();

        let (tx, rx) = oneshot::channel::<ClientResult>();
        let payload = Payload {
            url,
            method,
            body,
            headers,
            callback: tx,
        };

        let future = self.tx
            .clone()
            .send(payload)
            .map_err(|err| {
                Error::BadRequest(format!(
                    "Unexpected error sending http client request params to channel: {}",
                    err
                ))
            })
            .and_then(|_| {
                rx.map_err(|err| {
                    Error::BadRequest(format!(
                        "Unexpected error receiving http client response from channel: {}",
                        err
                    ))
                })
            })
            .and_then(|result| result)
            .map_err(move |err| {
                error!("{} {} : {}", method_clone, url_clone, err.to_string());
                err
            });

        Box::new(future)
    }
}

struct Payload {
    pub url: String,
    pub method: hyper::Method,
    pub body: Option<String>,
    pub headers: Option<hyper::Headers>,
    pub callback: oneshot::Sender<ClientResult>,
}