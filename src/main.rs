#![feature(try_trait)]

pub extern crate rusqlite;
pub extern crate notify;
pub extern crate walkdir;
pub extern crate chrono;

mod backend;

fn main() {
    let cfg = backend::TifariConfig::new(
        backend::DbOpenType::FromPath("db.sqlite".to_string()),
        "images".to_string());

    let backend = backend::TifariBackend::new(cfg).unwrap();
}
