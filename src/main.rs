extern crate backend;
use backend::*;

fn main() {
    let cfg = TifariConfig::new(
        "db.sqlite".to_string(),
        "images".to_string());

    let backend = TifariBackend::new(cfg).unwrap();
}
