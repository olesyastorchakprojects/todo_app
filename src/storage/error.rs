use thiserror::Error;

pub use super::sled::error::SledStorageError;
use strum_macros::AsRefStr;

#[derive(Error, Debug, AsRefStr)]
pub enum StorageError {
    #[error("Not found")]
    NotFound,

    #[error("No content")]
    NoContent,

    #[error("Failed to parse id from string")]
    ParseIdFromString(#[from] uuid::Error),

    #[error("Failed to create expiration date for session")]
    InvalidTtl,

    #[error("Session expired")]
    SessionExpired,

    #[error("Internal storage error")]
    Internal(#[source] SledStorageError),

    #[error("Blocking task join error")]
    JoinError(#[from] tokio::task::JoinError),
}
