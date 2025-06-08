use std::{net::SocketAddr, path::PathBuf};

use serde::Deserialize;
use strum_macros::AsRefStr;

#[derive(Debug, Deserialize, Copy, Clone, AsRefStr)]
#[serde(rename_all = "lowercase")]
pub enum StorageKind {
    Sled,
    Postgres,
    RocksDb,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct StorageSettings {
    pub backend: StorageKind,
    pub sled: Option<SledConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SledConfig {
    pub path: PathBuf,
    pub delete_batch_size: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryConfig {
    pub tracing_endpoint: String,
    pub tracing_sampling_rate: f64,
    pub metrics_endpoint: String,
    pub stdout_tracing: bool,
    pub tracing: bool,
    pub metrics: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub access_token_ttl_sec: i64,
    pub refresh_token_ttl_sec: i64,
    pub session_ttl_sec: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub addr: SocketAddr,
}

#[derive(Debug, Deserialize, Copy, Clone, AsRefStr)]
#[serde(rename_all = "lowercase")]
pub enum KDFKind {
    Argon2,
    Pbkdf2,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Argon2Config {
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pbkdf2Config {
    pub iterations: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct AuthSettings {
    pub kdf_algo: KDFKind,
    pub argon2: Option<Argon2Config>,
    pub pbkdf2: Option<Pbkdf2Config>,
    pub(crate) admins: Vec<AdminCredentials>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Limits {
    pub cells_per_second: u32,
    pub burst_per_second: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimits {
    pub global: Limits,
    pub per_ip: Limits,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimiterSettings {
    pub x_forwarded_for: bool,
    pub registration: RateLimits,
    pub login: RateLimits,
    pub admin: RateLimits,
    pub crud_light: RateLimits,
    pub crud_heavy: RateLimits,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminCredentials {
    pub email: String,
    pub password: String,
}
