#[macro_use]
extern crate stdweb;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate backend;

fn main() {
    
    let cfg = backend::TifariConfig::new(
        backend::DbOpenType::FromPath("db.sqlite".to_string()),
        "images".to_string());

    let backend = backend::TifariBackend::new(cfg).unwrap();

    run(&backend);
}

fn run(backend: &backend::TifariBackend)
{
    stdweb::initialize();
    stdweb::web::window().alert("Hello, world!");
    stdweb::event_loop();
}
