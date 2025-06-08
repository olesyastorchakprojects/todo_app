use std::str::Utf8Error;

use sled::transaction::TransactionError;
use strum_macros::AsRefStr;
use thiserror::Error;

use crate::storage::StorageError;

#[derive(Error, Debug, AsRefStr)]
pub enum SledStartupError {
    #[error("Failed to open sled storage")]
    OpenSledStorageError(#[source] sled::Error),
}

#[derive(Error, Debug, AsRefStr)]
pub enum SledStorageError {
    #[error("Data for key not found")]
    NotFound,

    #[error("Content for key not found")]
    NoContent,

    #[error("Failed to encode data")]
    Encode(#[from] bincode::error::EncodeError),

    #[error("Failed to decode data")]
    Decode(#[from] bincode::error::DecodeError),

    #[error("Failed to convert to utf8")]
    Conversion(#[from] Utf8Error),

    #[error("Failed to parse enum from string")]
    Strum(#[from] strum::ParseError),

    #[error("Sled error")]
    Sled(#[from] sled::Error),

    #[error("Key without prefix: {0}")]
    InvalidKey(String),

    #[error("TreeScan used without either 'within' or 'until_pagination' parameters or both: {0}")]
    UninitializedTreeScan(&'static str),

    #[error("sled transaction error")]
    Transaction(#[from] TransactionError),

    #[error("sled unabortable transaction error")]
    UnabortableTransaction(#[from] sled::transaction::UnabortableTransactionError),

    #[error("sled conflictable transaction error")]
    ConflicatableTransaction(#[from] sled::transaction::ConflictableTransactionError),
}

impl From<SledStorageError> for sled::transaction::ConflictableTransactionError<SledStorageError> {
    fn from(value: SledStorageError) -> Self {
        sled::transaction::ConflictableTransactionError::Abort(value)
    }
}

impl From<sled::transaction::TransactionError<SledStorageError>> for SledStorageError {
    fn from(value: sled::transaction::TransactionError<SledStorageError>) -> Self {
        match value {
            sled::transaction::TransactionError::Abort(e) => e,
            sled::transaction::TransactionError::Storage(e) => Self::Sled(e),
        }
    }
}

impl From<SledStorageError> for StorageError {
    fn from(value: SledStorageError) -> Self {
        match value {
            SledStorageError::NotFound => {
                tracing::warn!(error = ?value, error_type = %value.as_ref(), "Record not found by id");
                Self::NotFound
            }
            SledStorageError::NoContent => {
                tracing::warn!(error = ?value, error_type = %value.as_ref(), "No content for id");
                Self::NoContent
            }
            _ => {
                tracing::error!(error = ?value, error_type = %value.as_ref(), "Storage error");
                Self::Internal(value)
            }
        }
    }
}
