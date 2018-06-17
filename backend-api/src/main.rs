extern crate tifari_backend_api;
use tifari_backend_api::*;

fn main() {
    let addr = "127.0.0.1:8001".parse().unwrap();
    let mut db = backend::TifariDb::new(get_cfg()).unwrap();
    db.reload_root();

    let server = hyper::server::Http::new().bind(&addr, || { 
        let cfg = get_cfg();
        let staticfile = hyper_staticfile::Static::new(std::path::Path::new(cfg.get_root()));

        let service = Search::new(cfg, staticfile);
        Ok(service)
    }).unwrap();
    server.run().unwrap();
}
