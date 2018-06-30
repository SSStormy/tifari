extern crate tifari_backend_api;
use tifari_backend_api::*;

use std::sync::{Arc, RwLock};



fn main() {
    let addr = "127.0.0.1:8001".parse().unwrap();

    let cfg = Arc::new(RwLock::new(get_cfg()));
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
