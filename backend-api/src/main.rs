extern crate backend;
extern crate hyper;
extern crate futures;

#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate serde;

extern crate models;

extern crate hyper_staticfile;

use futures::future::{FutureResult, ok, err};
use futures::{Future, Stream, IntoFuture};

use hyper::header::{AccessControlAllowOrigin, AccessControlAllowMethods, ContentLength};
use hyper::server::{Service, Request, Response, Http};
use hyper::{Method, StatusCode};
use std::error::Error;

mod error;
use self::error::*;

struct Search {
    config: backend::TifariConfig,
    staticfile: hyper_staticfile::Static,
}

fn conv_result<T, EIn, EOut: From<EIn>>(what: Result<T, EIn>) -> Result<T, EOut> {
    match what {
        Ok(v) => Ok(v),
        Err(e) => Err(EOut::from(e)),
    }
}

fn req_to_json<'a, T>(req: hyper::Request) -> impl Future<Item=T, Error=APIError>
    where T: serde::de::DeserializeOwned {

    req.body()
        .map_err(APIError::Hyper)
        .concat2()
        .and_then(move |body: hyper::Chunk| {
            conv_result(serde_json::from_slice::<T>(&body))
        })
}

impl Service for Search {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;

    type Future = Box<Future<
        Item = Self::Response,
        Error = Self::Error>>;


    fn call(&self, req: Request) -> Self::Future {
        println!("Received request.");
        // TODO: @HACK cloning the config with each request is horrible
        let cfg = self.config.clone();

        // TODO : @HACK, we shouldn't have to box task1
        let task1: Box<Future<Item=Self::Response, Error=APIError>> = match (req.method(), req.path()) {
            (Method::Post, "/search") => {
                Box::new(req_to_json::<models::SearchQuery>(req)
                    .and_then(move |query| {
                        // TODO : pool db connections
                        // TODO : honestly just learn diesel
                        match backend::TifariDb::new_from_cfg(cfg) {
                            Ok(db) => Ok((query, db)),
                            Err(e) => Err(APIError::from(e)),
                       }
                    })
                    .and_then(|(query, db)| {
                         conv_result(db.search(&query.get_tags(), query.get_offset(), query.get_max()))
                    })
                    .and_then(|images| {
                        conv_result(serde_json::to_string(&models::SearchResult::new(images)))
                    })
                    .and_then(|payload| {
                        let response = Self::Response::new()
                            .with_status(StatusCode::Ok)
                            .with_header(ContentLength(payload.len() as u64))
                            .with_body(payload);

                        ok(response)
                }))
            },
            (_, _) => {
                println!("Redirecting to staticfile.");

                Box::new(self.staticfile.call(req)
                .then(|result: Result<hyper::Response, hyper::Error>| {
                    match result {
                        Ok(resp) => ok(resp),
                        Err(e) => err(APIError::from(e)),
                    }
                }))
            },
        };
        
        let finalized = task1.then(|result: Result<Self::Response, APIError>| {
            ok(match result {
                Ok(resp) => resp,
                Err(e) => { println!("Error: {:?}", e); Self::Response::new()
                    .with_status(StatusCode::InternalServerError)
                    .with_header(ContentLength(e.description().len() as u64))
                    .with_body(e.description().to_string()) },
            })
        })
            .and_then(|req| {
                ok(req.with_header(AccessControlAllowMethods(vec![Method::Get]))
                   .with_header(AccessControlAllowOrigin::Any))
                });
        
        Box::new(finalized)
    }
}

fn get_cfg() -> backend::TifariConfig{
    backend::TifariConfig::new(
        backend::DbOpenType::FromPath("db.sqlite".to_string()),
        "images".to_string())
}

fn setup_test_db() {
    let mut db = backend::TifariDb::new_from_cfg(get_cfg()).unwrap();
    let boat1 = "boat1.jpg".to_string();
    let boat2 = "boat2.jpg".to_string();
    let boat3 = "boat3.jpg".to_string();
    let boat4 = "boat4.jpg".to_string();

    let boat_tag = "boat".to_string();
    let spec_tag = "special".to_string();

    db.try_insert_image(&boat1).unwrap();
    db.try_insert_image(&boat2).unwrap();
    db.try_insert_image(&boat3).unwrap();
    db.try_insert_image(&boat4).unwrap();

    db.give_tag(&boat1, &boat_tag).unwrap();
    db.give_tag(&boat2, &boat_tag).unwrap();
    db.give_tag(&boat3, &boat_tag).unwrap();
    db.give_tag(&boat4, &boat_tag).unwrap();

    db.give_tag(&boat1, &spec_tag).unwrap();
    db.give_tag(&boat2, &spec_tag).unwrap();
}

fn main() {
    //setup_test_db();
    let addr = "127.0.0.1:8001".parse().unwrap();
    let server = Http::new().bind(&addr, || { 
        // TODO : this gets called on every fucking request
        let cfg = get_cfg();
        let staticfile = hyper_staticfile::Static::new(std::path::Path::new(cfg.get_root()));

        let service = Search { config: cfg, staticfile };
        Ok(service)
    }).unwrap();
    server.run().unwrap();
}
