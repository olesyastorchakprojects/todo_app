mod app;
mod config;
pub(crate) mod handlers;
mod init;
pub(crate) mod middleware;
pub(crate) mod service;
pub(crate) mod storage;
pub(crate) mod utils;

mod docs;

pub use config::Settings;
pub use handlers::error::AppError;
pub use init::StartupError;

use axum::Router;
use opentelemetry_sdk::{metrics::SdkMeterProvider, trace::SdkTracerProvider};

#[cfg(feature = "integration_tests")]
pub use app::build_app;

#[cfg(feature = "integration_tests")]
pub use init::init_storage;

#[cfg(feature = "integration_tests")]
pub use storage::{Session, SessionId, Todo, TodoId, User, UserId};

#[cfg(feature = "integration_tests")]
pub use service::Service;

#[cfg(feature = "integration_tests")]
pub use storage::test_util::TestStorageBuilder;

#[cfg(feature = "integration_tests")]
pub use handlers::types::{TodosPageResponse, UsersPageResponse};

#[cfg(feature = "integration_tests")]
pub use middleware::auth::AuthError;

#[cfg(feature = "integration_tests")]
pub use init::init_tracer_provider;
use tracing::{info, instrument};

pub struct TracingProviderGuard {
    provider: SdkTracerProvider,
}

impl TracingProviderGuard {
    pub fn new(settings: &Settings) -> Result<Self, StartupError> {
        Ok(Self {
            provider: init::init_tracer_provider(settings)?,
        })
    }
}

impl Drop for TracingProviderGuard {
    fn drop(&mut self) {
        let _ = self.provider.shutdown();
    }
}

pub struct MetricsProviderGuard {
    provider: SdkMeterProvider,
}

impl MetricsProviderGuard {
    pub fn new(settings: &Settings) -> Result<Self, StartupError> {
        Ok(Self {
            provider: init::init_metrics_provider(settings)?,
        })
    }
}

impl Drop for MetricsProviderGuard {
    fn drop(&mut self) {
        let _ = self.provider.shutdown();
    }
}

#[instrument(name = "init_app", skip_all)]
pub async fn init_app(settings: Settings) -> Result<(Router, service::Service), StartupError> {
    info!(settings = ?settings, "init_app with settings");

    let service = init::init_storage(&settings).await?;

    Ok((app::build_app(service.clone(), settings), service))
}
