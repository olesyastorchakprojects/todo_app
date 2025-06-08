mod observability;
mod storage;

use crate::{handlers::error::AppError, storage::SledStartupError};
use thiserror::Error;

pub use observability::{init_metrics_provider, init_tracer_provider};
pub use storage::init_storage;

#[derive(Debug, Error)]
pub enum StartupError {
    #[error("Environment variable not set: {0}")]
    FailedToLoadEnvVar(&'static str),

    #[error("Failed to open sled storage")]
    OpenSledStorage(#[from] SledStartupError),

    #[error("Failed to create admins")]
    CreateAdmins(#[from] AppError),

    #[error("Unsupported storage kind: {0}")]
    UnsupportedStorage(String),

    #[error("Missing storage config: {0}")]
    MissingStorageConfig(String),

    #[error("Failed to load configs")]
    LoadConfig(#[from] config::ConfigError),

    #[error("Failed to init tracing")]
    InitTracing(#[from] opentelemetry_otlp::ExporterBuildError),

    #[error("Failed to set global tracing provider")]
    SetGlobalTracingProvider(#[from] tracing::subscriber::SetGlobalDefaultError),
}
