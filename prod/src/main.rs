extern crate tifari_backend_api;
use tifari_backend_api::*;
use std::thread;

fn main() {

    let cfg = get_cfg();
    let frontend_addr = cfg.get_frontend_address().clone();

    let api_thread = thread::spawn(move || {
        println!("Serving API at {}", cfg.get_api_address());
        run_server(cfg);
    });

    let addr = frontend_addr.parse().unwrap();
    let server = hyper::server::Http::new().bind(&addr, || { 
        let mut cwd = std::env::current_dir().unwrap();
        cwd.push("static");

        let staticfile = hyper_staticfile::Static::new(cwd);
        Ok(staticfile)
    }).unwrap();

    println!("Serving static frontend at {}" ,frontend_addr);
    server.run().unwrap();
    api_thread.join().unwrap();
}
