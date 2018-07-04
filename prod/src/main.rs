extern crate tifari_backend_api;
extern crate open;
extern crate ctrlc;

use tifari_backend_api::*;
use std::thread;

const LOCK_FILE: &'static str = "tifari_lock";

fn main() {
    
    ctrlc::set_handler(|| {
        match std::fs::remove_file(LOCK_FILE) {
            Ok(()) => {},
            Err(_e) => println!("Failed to remove lock file."),
        }
        std::process::exit(0);

    }).expect("Error setting Ctrl-C handler");

    if std::fs::metadata(LOCK_FILE).is_ok() {
        println!("LOCK_FILE: {} found!", LOCK_FILE);
        println!("We think that an instance of tifari is already running.");
        println!("If you are sure this is not the case, delete tifari_lock from {}", std::env::current_dir().unwrap().as_path().to_string_lossy());

        std::process::exit(0);
    }

    std::fs::File::create(LOCK_FILE).unwrap();

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
    let url = format!("http://{}", frontend_addr);
    open::that(url).unwrap();

    server.run().unwrap();
    api_thread.join().unwrap();
}
