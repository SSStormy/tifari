pub extern crate backend;
pub extern crate hyper;
pub extern crate futures;
pub extern crate serde_json;
pub extern crate serde;
pub extern crate models;
pub extern crate hyper_staticfile;

use futures::future::{FutureResult, ok, err};
use futures::{Future, Stream};

use hyper::header::{AccessControlAllowOrigin, AccessControlAllowMethods, ContentLength};
use hyper::server::{Service, Request, Response};
use hyper::{Method, StatusCode};
use std::error::Error;

pub mod error;
use self::error::*;
use std::sync::{Arc, RwLock};

pub struct Search {
    config: Arc<RwLock<backend::TifariConfig>>,
    staticfile: Arc<hyper_staticfile::Static>,
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

pub struct APINewService {
    config: Arc<RwLock<backend::TifariConfig>>,
    staticfile: Arc<hyper_staticfile::Static>,
}

impl hyper::server::NewService for APINewService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Instance = Search;

    fn new_service(&self) -> Result<Self::Instance, std::io::Error> {
        Ok(Search{config: self.config.clone(), staticfile: self.staticfile.clone()})
    }
}

impl APINewService {
    pub fn new(config: Arc<RwLock<backend::TifariConfig>>, staticfile: Arc<hyper_staticfile::Static>) -> Self {
        APINewService { config, staticfile }
    }
}

pub enum ConfigStatus {
    Valid = 0,
    InvalidImageFolder = 1,
    ImageFolderIsNotAFolder = 2,
}

impl Service for Search {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;

    type Future = Box<Future<
        Item = Self::Response,
        Error = Self::Error>>;


    fn call(&self, req: Request) -> Self::Future {
        println!("Received request. {}", req.path());

        // TODO : @HACK, we shouldn't have to box task1
        let cfg = self.config.clone();
        let task1: Box<Future<Item=Self::Response, Error=APIError>> = match (req.method(), req.path()) {
            (Method::Get, "/api/v1/config_status") => {

                let get_status = || {
                    let metadata = match std::fs::metadata(cfg.read().unwrap().get_root()) {
                        Ok(v) => v,
                        Err(_) => return ConfigStatus::InvalidImageFolder,
                    };

                    if !metadata.is_dir() { return ConfigStatus::ImageFolderIsNotAFolder }

                    ConfigStatus::Valid
                };

                let payload = format!("{{\"state\": {}}}", get_status() as i32);

                Box::new(ok(get_resp_with_payload(payload)))
            },

            (Method::Get, "/api/v1/tag_queue_size") => {
                let get_response = || {
                    let db = backend::TifariDb::new(cfg)?;
                    let num = db.get_num_elements_in_tag_queue()?;
                    let payload = serde_json::to_string(&models::TagQueueSizeResponse::new(num))?;
                    Ok(get_resp_with_payload(payload))
                };

                Box::new(FutureResult::from(get_response()))
            },
            (Method::Post, "/api/v1/remove_tags") => {
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
            (Method::Get, "/api/v1/get_all_tags") => {
                let get_response = || {
                    let db = backend::TifariDb::new(cfg)?;
                    let tags = db.get_all_tags()?;
                    let payload = serde_json::to_string(&tags)?;

                    Ok(get_resp_with_payload(payload))
                };

                Box::new(FutureResult::from(get_response()))
            },
            (Method::Post, "/api/v1/add_tags") => {
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
                            for id in query.get_image_ids() {
                                match db.give_tag(*id, &tag) {
                                    Ok(id) => tags.push(models::Tag::new(id, tag.clone())),
                                    Err(e) => println!("add_tags req failed for tag {}. {:?}", tag,  e),
                                };
                            }
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
            (Method::Get, "/api/v1/config") => {

                let cfg_lock = cfg.read().unwrap();
                let cfg = &(*cfg_lock); // unwraps the lock

                let payload = serde_json::to_string(&cfg).unwrap();
                Box::new(ok(get_resp_with_payload(payload)))
            },
            (Method::Post, "/api/v1/config") => {
                let cfg1 = cfg.clone();
                let cfg2 = cfg.clone();
                let res = req_to_json::<std::collections::HashMap<String, String>>(req)
                     .and_then(move |patch| {
                         // modify the config
                         let mut cfg = cfg1.write().unwrap();
                         cfg.update(patch);

                         ok(())
                     })
                     .and_then(move |()| {
                         let cfg_lock = cfg2.read().unwrap();
                         let cfg = &(*cfg_lock); // unwraps the lock

                         conv_result(serde_json::to_string(&cfg))
                     })
                     .and_then(move |payload| {
                        conv_result(std::fs::write("config.json", &payload))
                     })
                     .and_then(move |()| {
                         ok(get_default_success_response())
                     });

                Box::new(res)
            },
            (Method::Get, "/api/v1/reload") => {

                let get_response = || {
                    let mut db = backend::TifariDb::new(cfg.clone())?;
                    db.reload_root(cfg.read().unwrap().get_root());
                    Ok(get_default_success_response())
              };

                Box::new(FutureResult::from(get_response()))
            },
            
            (Method::Get, "/api/v1/tag_queue") => {

                let get_response = || {
                    let db = backend::TifariDb::new(cfg)?;
                    let queue = db.get_tag_queue()?;
                    let payload = serde_json::to_string(&queue)?;
                    Ok(get_resp_with_payload(payload))
                };

                Box::new(FutureResult::from(get_response()))
            },
            (Method::Post, "/api/v1/search") => {
                Box::new(req_to_json::<Vec<String>>(req)
                    .and_then(move |query| {
                        match backend::TifariDb::new(cfg) {
                            Ok(db) => Ok((query, db)),
                            Err(e) => Err(APIError::from(e)),
                       }
                    })
                    .and_then(|(query, db)| {
                        let query_str: Vec<&str> = query.iter().map(|e| &e[..]).collect();
                        conv_result(db.search(&query_str))
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

fn get_default_cfg<T>(e: T) -> backend::TifariConfig 
    where T: std::error::Error {

    println!("Using the default config due to error when reading config.json.");
    println!("Error: {:?}", e);
    backend::TifariConfig::default()
}

pub fn get_cfg() -> backend::TifariConfig {
    let cfg = match std::fs::read_to_string("config.json") {
        Ok(v) => v,
        Err(e) => return get_default_cfg(e),
    };
    
    match serde_json::from_str(&cfg) {
        Ok(v) => v,
        Err(e) => get_default_cfg(e),
    }
}

pub fn run_server(config: backend::TifariConfig) {

    let addr = config.get_api_address().parse().unwrap();

    let cfg = Arc::new(RwLock::new(config));
    let mut db = backend::TifariDb::new(cfg.clone()).unwrap();
    {
        let cfg = cfg.clone();
        db.reload_root(cfg.read().unwrap().get_root());
    }

    let staticfile = Arc::new(hyper_staticfile::Static::new(std::path::Path::new(cfg.read().unwrap().get_root())));

    let service = APINewService::new(cfg, staticfile);

    let server = hyper::server::Http::new().bind(&addr, service).unwrap();

    server.run().unwrap();
}
