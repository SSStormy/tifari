use super::*;

#[derive(Debug)]
pub enum APIError {
    Hyper(hyper::Error),
    FromUtf8(std::string::FromUtf8Error),
    Json(serde_json::Error),
    IO(std::io::Error),
    Backend(backend::Error),
}

impl std::error::Error for APIError {
    fn description(&self) -> &str {
        match self {
            APIError::Hyper(e) => e.description(),
            APIError::FromUtf8(e) => e.description(),
            APIError::Json(e) => e.description(),
            APIError::IO(e) => e.description(),
            APIError::Backend(_) => "backend error",
        }
    }
}
impl From<std::io::Error> for APIError {
    fn from(e: std::io::Error) -> Self { APIError::IO(e) }
}

impl From<backend::Error> for APIError {
    fn from(e: backend::Error) -> Self { APIError::Backend(e) }
}

impl std::fmt::Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"Error")
    }
}

impl std::convert::From<hyper::Error> for APIError {
    fn from(e: hyper::Error) -> Self { APIError::Hyper(e) }
}

impl From<std::string::FromUtf8Error> for APIError {
    fn from(e: std::string::FromUtf8Error) -> Self { APIError::FromUtf8(e) }
}

impl From<serde_json::Error> for APIError {
    fn from(e: serde_json::Error) -> Self { APIError::Json(e) }
}

