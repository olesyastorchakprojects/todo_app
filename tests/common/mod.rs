#![allow(dead_code, unused_imports)]

mod client;
mod server;

use std::sync::Arc;

use axum::Router;
pub use client::TestAppClient;
use todo_app::Service;
use todo_app::{build_app, Settings};

pub use server::{spawn_test_app, TestAppHandle};
use todo_app::TestStorageBuilder;

#[derive(Debug, serde::Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct UnauthorizedBody {
    pub error: String,
    pub message: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateTodoResponse(pub String);

pub async fn create_test_app(settings_file: Option<&str>) -> Router {
    let todo_storage = TestStorageBuilder::new().build_todo().await;
    let user_storage = TestStorageBuilder::new().build_user().await;
    let session_storage = TestStorageBuilder::new().build_session().await;
    let flush_storage = TestStorageBuilder::new().build_flush().await;

    let settings = match settings_file {
        Some(file_name) => Settings::from_file(file_name).unwrap(),
        None => Settings::new().unwrap(),
    };

    let service = Service::new(todo_storage, user_storage, session_storage, flush_storage).await;
    service.user().create_admins(&settings).await.unwrap();

    build_app(service, settings)
}
