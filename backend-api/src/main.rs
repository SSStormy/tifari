extern crate backend;
extern crate hyper;
extern crate futures;

#[macro_use]
extern crate serde_derive;

extern crate serde;
#[macro_use]
extern crate serde_json;

use futures::future::{FutureResult, ok, err};
use futures::{Future, Stream, IntoFuture};

use hyper::header::ContentLength;
use hyper::server::{Service, Request, Response, Http};
use hyper::{Method, StatusCode};
use std::error::Error;

struct Search<'a> {
    backend: &'a backend::TifariBackend,
}

#[derive(Debug)]
pub enum APIError {
    Hyper(hyper::Error),
    FromUtf8(std::string::FromUtf8Error),
    Json(serde_json::Error),
}

impl std::error::Error for APIError {
    fn description(&self) -> &str {
        match self {
            APIError::Hyper(e) => e.description(),
            APIError::FromUtf8(e) => e.description(),
            APIError::Json(e) => e.description(),
        }
    }
}

impl std::fmt::Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"Error")
    }
}

impl std::convert::From<hyper::Error> for APIError {
    fn from(e: hyper::Error) -> Self { APIError::Hyper(e) }
}

impl From<std::string::FromUtf8Error> for APIError {
    fn from(e: std::string::FromUtf8Error) -> Self { APIError::FromUtf8(e) }
}

impl From<serde_json::Error> for APIError {
    fn from(e: serde_json::Error) -> Self { APIError::Json(e) }
}

#[derive(Deserialize)]
struct SearchQuery {
    tags: std::collections::HashSet<String>
}

fn conv_result<T, EIn, EOut: From<EIn>>(what: Result<T, EIn>) -> Result<T, EOut> {
    match what {
        Ok(v) => Ok(v),
        Err(e) => Err(EOut::from(e)),
    }
}

impl<'a> Service for Search<'a> {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;

    type Future = Box<Future<
        Item = Self::Response,
        Error = Self::Error>>;


    fn call(&self, req: Request) -> Self::Future {

        let response = Self::Response::new();

        let task = req.body()
            .map_err(APIError::Hyper)
            .fold(Vec::new(), |mut acc, chunk| -> FutureResult<Vec<u8>, APIError> {
                acc.extend_from_slice(&*chunk);
                ok(acc)
            })
            .and_then(|v| {
                conv_result(String::from_utf8(v))
            })
            .and_then(|s| {
                conv_result(serde_json::from_str::<SearchQuery>(&s))
            })
            .then(|result| {
                ok(match result {
                    Ok(s) => response
                                .with_status(StatusCode::Ok)
                                .with_header(ContentLength(2 as u64))
                                .with_body("OK"),
                    Err(e) => response
                                .with_status(StatusCode::InternalServerError)
                                .with_header(ContentLength(e.description().len() as u64))
                                .with_body(e.description().to_string()),
                })
            });

        Box::new(task)

        /*
        match (req.method(), req.path()) {
            (Method::Get, "/image") => {
                response.set_status(StatusCode::Ok);
                response.set_body(req.body());
            },
            /*
            (Method::Get, "/search") => {
            },
            (Method::Post, "/image") => {
            }
            (Method::Get, "/tag_queue") => {
            }
            */
            _ => {
                response.set_status(StatusCode::NotFound);
            },
        }

    */
  //      Box::new(futures::future::ok(response))
    }
}

fn main() {

    let cfg = backend::TifariConfig::new(
        backend::DbOpenType::FromPath("db.sqlite".to_string()),
        "images".to_string());

    let backend = backend::TifariBackend::new(cfg).unwrap();

    let addr = "127.0.0.1:8001".parse().unwrap();
    let server = Http::new().bind(&addr, move || Ok(Search {backend: &backend})).unwrap();
    server.run().unwrap();
}
