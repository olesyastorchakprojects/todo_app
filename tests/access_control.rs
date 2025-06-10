mod common;
use common::{create_test_app, spawn_test_app, CreateTodoResponse, TestAppClient};
use reqwest::StatusCode;

#[tokio::test]
async fn access_todo_without_token() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let res = client.create_todo(None, None).await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn access_other_user_todo() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    // user A
    let tokens = client.register_and_login("userA@gmail.com", "123").await;

    let res = client.create_todo(Some(&tokens.access_token), None).await;
    let todo_id = res.json::<CreateTodoResponse>().await.unwrap().0;

    // user B
    let tokens = client.register_and_login("userB@gmail.com", "123").await;

    let res = client.get_todo(&tokens.access_token, &todo_id).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
