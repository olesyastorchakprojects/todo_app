mod common;
use common::{create_test_app, spawn_test_app, LoginResponse, TestAppClient};
use reqwest::StatusCode;

#[tokio::test]
async fn register_success() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let res = client.register_user("aaa@gmail.com", "123").await;

    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn register_duplicate_email() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let res = client.register_user("aaa@gmail.com", "123").await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let res = client.register_user("aaa@gmail.com", "123").await;

    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn login_success() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let res = client.register_user("aaa@gmail.com", "123").await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let res = client.login_user("aaa@gmail.com", "123").await;

    assert_eq!(res.status(), StatusCode::OK);

    let token = res.json::<LoginResponse>().await.unwrap().access_token;
    assert_ne!(token.len(), 0);
}

#[tokio::test]
async fn login_wrong_password() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let res = client.register_user("aaa@gmail.com", "123").await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let res = client.login_user("aaa@gmail.com", "xxx").await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_nonexistent_user() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let res = client.login_user("aaa@gmail.com", "xxx").await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn register_with_missing_fields() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let res = client.register_user("", "123").await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let res = client.register_user("aaa", "").await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let res = client.register_user("", "").await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn login_with_missing_fields() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let res = client.login_user("", "123").await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let res = client.login_user("aaa", "").await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let res = client.login_user("", "").await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
