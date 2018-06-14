use super::*;

pub type Result<T> = std::result::Result<T, BackendError>;
pub type Error = BackendError;

#[derive(Debug)]
pub enum BackendError 
{
    SQLite(rusqlite::Error),
    IO(std::io::Error),
    Opt(std::option::NoneError),
    Notify(notify::Error),
    ImageExists,
    NoChangesOccured,
    BadTag,
}

impl From<std::io::Error> for BackendError 
{
    fn from(error: std::io::Error) -> Self { BackendError::IO(error) }
}

impl From<rusqlite::Error> for BackendError 
{
    fn from(error: rusqlite::Error) -> Self { BackendError::SQLite(error) }
}

impl From<std::option::NoneError> for BackendError
{
    fn from(error: std::option::NoneError) -> Self { BackendError::Opt(error) }
}

impl From<notify::Error> for BackendError
{
    fn from(error: notify::Error) -> Self { BackendError::Notify(error) }
}
