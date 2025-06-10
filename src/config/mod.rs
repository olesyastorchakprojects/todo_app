pub(crate) mod types;

use std::net::SocketAddr;

use config::{Config, Environment, File};
use serde::Deserialize;
use types::{AuthSettings, RateLimiterSettings};
pub(crate) use types::{JwtConfig, ServerConfig, StorageSettings, TelemetryConfig};

use crate::{init::StartupError, trace_err, utils::JWT_SECRET_KEY};

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub(crate) storage: StorageSettings,
    pub(crate) telemetry: TelemetryConfig,
    pub(crate) jwt: JwtConfig,
    pub(crate) server: ServerConfig,
    pub(crate) auth: AuthSettings,
    pub(crate) rate_limiter: RateLimiterSettings,
}

impl Settings {
    pub fn new() -> Result<Self, StartupError> {
        dotenv::dotenv().ok();

        let run_mode = std::env::var("RUN_MODE").unwrap_or("development".into());

        Settings::from_file(&run_mode)
    }

    pub fn from_file(file_name: &str) -> Result<Self, StartupError> {
        std::env::var(JWT_SECRET_KEY)
            .map_err(|_| StartupError::FailedToLoadEnvVar(JWT_SECRET_KEY))?;

        trace_err!(
            Config::builder()
                .add_source(File::with_name("config/default"))
                .add_source(File::with_name(&format!("config/{file_name}")).required(false))
                .add_source(Environment::with_prefix("APP").separator("__"))
                .build()?
                .try_deserialize(),
            "failed to build app settings"
        )
        .map_err(Into::into)
    }

    pub fn server_addr(&self) -> SocketAddr {
        self.server.addr
    }

    pub fn tracing_enabled(&self) -> bool {
        self.telemetry.tracing
    }

    pub fn metrics_enabled(&self) -> bool {
        self.telemetry.metrics
    }
}
