use crate::storage::StorageError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use strum_macros::AsRefStr;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Error, AsRefStr, ToSchema)]
#[strum(serialize_all = "snake_case")]
pub enum AppError {
    #[error("Not found")]
    NotFound,

    #[error("No content")]
    NoContent,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Password mismatch")]
    PasswordMismatch,

    #[error("User by email not found")]
    UserByEmailNotFound,

    #[error("Forbidden")]
    Forbidden,

    #[schema(value_type = String)]
    #[error("Encoding token failed")]
    EncodingToken(#[from] jsonwebtoken::errors::Error),

    #[schema(value_type = String)]
    #[error("Hasing password failed")]
    HashingPassword(#[from] argon2::password_hash::Error),

    #[error("Missing pbkdf2 config")]
    MissingPbkdf2Config,

    #[error("Missing argon2 config")]
    MissingArgon2Config,

    #[schema(value_type = String)]
    #[error("Invalid argon2 config")]
    InvalidArgon2Config(#[from] argon2::Error),

    #[schema(value_type = String)]
    #[error("Parsing password hash failed")]
    ParsePasswordHash(#[from] std::string::FromUtf8Error),

    #[error("Invalid role")]
    InvalidRole(String),

    #[schema(value_type = String)]
    #[error("Internal storage error")]
    InternalStorage(#[source] StorageError),

    #[error("Environment variable not set: {0}")]
    FailedToLoadEnvVar(&'static str),

    #[error("Failed to generate expiration timestamp")]
    InvalidTtl,

    #[error("Email and password must be provided")]
    MissingPasswordEmail,

    #[error("Patch must not be empty")]
    EmptyPatch,

    #[schema(value_type = String)]
    #[error("Failed joining tokio task")]
    JoinTask(#[from] tokio::task::JoinError),
}

impl From<StorageError> for AppError {
    fn from(value: StorageError) -> Self {
        match value {
            StorageError::NotFound => Self::NotFound,
            StorageError::NoContent => Self::NoContent,
            _ => Self::InternalStorage(value),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!(error = ?self, "AppError");

        let status = match &self {
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,
            AppError::NoContent { .. } => return StatusCode::NO_CONTENT.into_response(),
            AppError::UserAlreadyExists { .. } => StatusCode::CONFLICT,
            AppError::UserByEmailNotFound { .. } => StatusCode::UNAUTHORIZED,
            AppError::PasswordMismatch { .. } => StatusCode::UNAUTHORIZED,
            AppError::Forbidden { .. } => StatusCode::FORBIDDEN,
            AppError::InvalidRole { .. }
            | AppError::MissingPasswordEmail
            | AppError::EmptyPatch => StatusCode::BAD_REQUEST,
            AppError::InternalStorage { .. }
            | AppError::EncodingToken { .. }
            | AppError::FailedToLoadEnvVar { .. }
            | AppError::InvalidTtl
            | AppError::HashingPassword { .. }
            | AppError::ParsePasswordHash { .. }
            | AppError::InvalidArgon2Config { .. }
            | AppError::MissingArgon2Config
            | AppError::MissingPbkdf2Config
            | AppError::JoinTask { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(json!({
            "error": self.as_ref(),
            "message": self.to_string(),
        }));
        (status, body).into_response()
    }
}
