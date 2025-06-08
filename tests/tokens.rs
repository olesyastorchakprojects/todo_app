#![allow(unused_imports)]
mod common;
use serial_test::{parallel, serial};
use std::time::Duration;

use common::{create_test_app, spawn_test_app, LoginResponse, TestAppClient, UnauthorizedBody};
use reqwest::StatusCode;
use todo_app::AuthError;

struct EnvSetter {
    name: &'static str,
}
impl EnvSetter {
    fn new(name: &'static str, value: &str) -> Self {
        std::env::set_var(name, value);
        Self { name }
    }
}

impl Drop for EnvSetter {
    fn drop(&mut self) {
        std::env::remove_var(self.name);
    }
}
#[tokio::test]
#[serial]
async fn expired_access_token_test() {
    let _env_setter = EnvSetter::new("APP__JWT__ACCESS_TOKEN_TTL_SEC", "-1");

    let handle = spawn_test_app(create_test_app(Some("test")).await).await;
    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("userA@gmail.com", "123").await;

    let response = client.create_todo(Some(&tokens.access_token), None).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = response.json::<UnauthorizedBody>().await.unwrap();
    assert_eq!(body.error, AuthError::TokenExpired.as_ref());
}

#[tokio::test]
#[serial]
async fn expired_refresh_token_test() {
    let _env_setter1 = EnvSetter::new("APP__JWT__ACCESS_TOKEN_TTL_SEC", "-1");
    let _env_setter2 = EnvSetter::new("APP__JWT__REFRESH_TOKEN_TTL_SEC", "-1");

    let handle = spawn_test_app(create_test_app(Some("test")).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("userA@gmail.com", "123").await;

    let response = client.refresh_token(&tokens.refresh_token).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = response.json::<UnauthorizedBody>().await.unwrap();
    assert_eq!(body.error, AuthError::TokenExpired.as_ref());
}

#[tokio::test]
#[serial]
async fn valid_refresh_token_test() {
    let _env_setter = EnvSetter::new("APP__JWT__ACCESS_TOKEN_TTL_SEC", "-1");
    let handle = spawn_test_app(create_test_app(Some("test")).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("userA@gmail.com", "123").await;

    let response = client.create_todo(Some(&tokens.access_token), None).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let response = client.refresh_token(&tokens.refresh_token).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[parallel]
async fn old_refresh_token_rejected_test() {
    let handle = spawn_test_app(create_test_app(Some("test")).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("userA@gmail.com", "123").await;

    let response = client.refresh_token(&tokens.refresh_token).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response = client.refresh_token(&tokens.refresh_token).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = response.json::<UnauthorizedBody>().await.unwrap();
    assert_eq!(body.error, AuthError::InavlidRefreshJti.as_ref());
}

#[tokio::test]
#[parallel]
async fn valid_refresh_and_access_token_test() {
    let handle = spawn_test_app(create_test_app(Some("test")).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("userA@gmail.com", "123").await;

    let response = client.refresh_token(&tokens.refresh_token).await;
    assert_eq!(response.status(), StatusCode::OK);

    let tokens = response.json::<LoginResponse>().await.unwrap();

    let response = client.create_todo(Some(&tokens.access_token), None).await;
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
#[parallel]
async fn logout_invalidates_tokens_test() {
    let handle = spawn_test_app(create_test_app(Some("test")).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("userA@gmail.com", "123").await;

    let response = client.create_todo(Some(&tokens.access_token), None).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let response = client.logout(&tokens.access_token).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = client.create_todo(Some(&tokens.access_token), None).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = response.json::<UnauthorizedBody>().await.unwrap();
    assert_eq!(body.error, AuthError::InvalidSession.as_ref());
}

#[tokio::test]
#[parallel]
async fn refresh_token_required_test() {
    let handle = spawn_test_app(create_test_app(Some("test")).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("userA@gmail.com", "123").await;
    let response = client.refresh_token(&tokens.access_token).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = response.json::<UnauthorizedBody>().await.unwrap();
    assert_eq!(body.error, AuthError::ExpectedRefreshToken.as_ref());
}

#[tokio::test]
#[parallel]
async fn access_token_required_test() {
    let handle = spawn_test_app(create_test_app(Some("test")).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("userA@gmail.com", "123").await;

    let response = client.create_todo(Some(&tokens.refresh_token), None).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = response.json::<UnauthorizedBody>().await.unwrap();
    assert_eq!(body.error, AuthError::ExpectedAccessToken.as_ref());
}
