extern crate tifari_backend_api;
use tifari_backend_api::*;
use std::thread;

fn main() {
    let api_thread = thread::spawn(move || {
        let addr = "127.0.0.1:8001".parse().unwrap();
        let mut db = backend::TifariDb::new(get_cfg()).unwrap();
        db.reload_root();

        let server = hyper::server::Http::new().bind(&addr, || { 
            let cfg = get_cfg();
            let staticfile = hyper_staticfile::Static::new(std::path::Path::new(cfg.get_root()));

            let service = Search::new(cfg, staticfile);
            Ok(service)
        }).unwrap();

        println!("Serving API");
        server.run().unwrap();

    });


    let addr = "127.0.0.1:3001".parse().unwrap();
    let server = hyper::server::Http::new().bind(&addr, || { 
        let mut cwd = std::env::current_dir().unwrap();
        cwd.push("static");

        let staticfile = hyper_staticfile::Static::new(cwd);
        Ok(staticfile)
    }).unwrap();

    println!("Serving static frontend");
    server.run().unwrap();
    api_thread.join().unwrap();
}
