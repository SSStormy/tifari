extern crate backend;
extern crate hyper;
extern crate futures;

extern crate serde_json;
extern crate serde;

extern crate models;

extern crate hyper_staticfile;

use futures::future::{FutureResult, ok, err};
use futures::{Future, Stream};

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

fn get_default_success_response() -> hyper::Response {
    let payload = "{\"status\": 200}";
    let response = hyper::Response::new() 
        .with_status(StatusCode::Ok)
        .with_header(ContentLength(payload.len() as u64))
        .with_body(payload);

    response
}

fn get_resp_with_payload(payload: String) -> hyper::Response {
    hyper::Response::new() 
        .with_status(StatusCode::Ok)
        .with_header(ContentLength(payload.len() as u64))
        .with_body(payload)
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
            (Method::Get, "/api/tag_queue_size") => {
                let get_response = || {
                    let db = backend::TifariDb::new(cfg.clone())?;
                    let num = db.get_num_elements_in_tag_queue()?;
                    let payload = serde_json::to_string(&models::TagQueueSizeResponse::new(num))?;
                    Ok(get_resp_with_payload(payload))
                };

                Box::new(FutureResult::from(get_response()))
            }
            (Method::Post, "/api/remove_tags") => {
               Box::new(req_to_json::<models::RemoveTagsRequest>(req)
                    .and_then(move |query| {
                        match backend::TifariDb::new(cfg) {
                            Ok(db) => Ok((query, db)),
                            Err(e) => Err(APIError::from(e)),
                       }
                    })
                    .and_then(|(query, mut db)| {

                        for tag in query.get_tag_ids() {
                            for img in query.get_image_ids() {
                                match db.remove_tag(*img, *tag) {
                                    Ok(()) => (),
                                    Err(e) => println!("remove tags failed: {:?}", e),
                                };
                            }
                        }
                        ok(get_default_success_response())
                    })
                )
            },
            (Method::Get, "/api/get_all_tags") => {
                let get_response = || {
                    let db = backend::TifariDb::new(cfg.clone())?;
                    let tags = db.get_all_tags()?;
                    let payload = serde_json::to_string(&tags)?;

                    Ok(get_resp_with_payload(payload))
                };

                Box::new(FutureResult::from(get_response()))
            },
            (Method::Post, "/api/add_tags") => {
                Box::new(req_to_json::<models::AddTagsRequest>(req)
                    .and_then(move |query| {
                        match backend::TifariDb::new(cfg) {
                            Ok(db) => Ok((query, db)),
                            Err(e) => Err(APIError::from(e)),
                       }
                    })
                    .and_then(|(query, mut db)| {
                        let mut tags = vec![];

                        for tag in query.get_tags() {
                            match db.give_tag(query.get_image_id(), &tag) {
                                Ok(id) => tags.push(models::Tag::new(id, tag.clone())),
                                Err(e) => println!("add_tags req failed for tag {}. {:?}", tag,  e),
                            };
                        }

                        ok(tags)
                    })
                    .and_then(|tags| {
                        conv_result(serde_json::to_string(&tags))
                    })
                    .and_then(|payload| {
                        ok(get_resp_with_payload(payload))
                }))
            }
            
            (Method::Get, "/api/reload") => {

                let get_response = || {
                    let mut db = backend::TifariDb::new(cfg.clone())?;
                    db.reload_root();
                    Ok(get_default_success_response())
              };

                Box::new(FutureResult::from(get_response()))
            },
            
            (Method::Get, "/api/tag_queue") => {

                let get_response = || {
                    let db = backend::TifariDb::new(cfg.clone())?;
                    let queue = db.get_tag_queue()?;
                    let payload = serde_json::to_string(&queue)?;
                    Ok(get_resp_with_payload(payload))
                };

                Box::new(FutureResult::from(get_response()))
            },
            (Method::Post, "/api/search") => {
                Box::new(req_to_json::<models::SearchQuery>(req)
                    .and_then(move |query| {
                        match backend::TifariDb::new(cfg) {
                            Ok(db) => Ok((query, db)),
                            Err(e) => Err(APIError::from(e)),
                       }
                    })
                    .and_then(|(query, db)| {
                         conv_result(db.search(&query.get_tags(), query.get_offset(), query.get_max()))
                    })
                    .and_then(|images| {
                        conv_result(serde_json::to_string(&images))
                    })
                    .and_then(|payload| {
                        ok(get_resp_with_payload(payload))
                }))
            },
            (_, _) => {
                println!("Redirecting to staticfile.");

                let result = self.staticfile
                    .call(req)
                    .then(|result: Result<hyper::Response, hyper::Error>| {
                        match result {
                            Ok(resp) => ok(resp),
                            Err(e) => err(APIError::from(e)),
                        }
                    });

                Box::new(result)
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

// TODO : proper config loading etc etc
fn get_cfg() -> backend::TifariConfig {
    let cfg = std::fs::read_to_string("config.json").unwrap();
    serde_json::from_str(&cfg).unwrap()
}

fn main() {
    let addr = "127.0.0.1:8001".parse().unwrap();
    let mut db = backend::TifariDb::new(get_cfg()).unwrap();
    db.reload_root();

    let server = Http::new().bind(&addr, || { 
        let cfg = get_cfg();
        let staticfile = hyper_staticfile::Static::new(std::path::Path::new(cfg.get_root()));

        let service = Search { config: cfg, staticfile };
        Ok(service)
    }).unwrap();
    server.run().unwrap();
}
