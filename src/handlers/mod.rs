pub(crate) mod admin;
pub(crate) mod auth;
pub(crate) mod error;
pub(crate) mod todo;
pub mod types;

pub(crate) use crate::service::Service;
use axum::{http::StatusCode, response::IntoResponse};
pub(crate) use types::*;

#[tracing::instrument(name = "health", skip_all)]
pub(crate) async fn health() -> impl IntoResponse {
    StatusCode::OK
}
