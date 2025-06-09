mod common;
use common::{create_test_app, spawn_test_app, CreateTodoResponse, TestAppClient};
use reqwest::StatusCode;
use todo_app::TodosPageResponse;
use todo_app::{Todo, TodoId};

#[tokio::test]
async fn create_and_get_todo() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("user@gmail.com", "123").await;

    let res = client.create_todo(Some(&tokens.access_token), None).await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let todo_id = res.json::<CreateTodoResponse>().await.unwrap().0;

    let res = client.get_todo(&tokens.access_token, &todo_id).await;
    assert_eq!(res.status(), StatusCode::OK);

    let todo = res.json::<Todo>().await.unwrap();
    assert_eq!(todo.id.to_string(), todo_id);
}

#[tokio::test]
async fn get_todos_paginated() {
    let todo_count = 15;
    let limit = 10;
    let mut todo_items = Vec::with_capacity(todo_count);

    let handle = spawn_test_app(create_test_app(None).await).await;
    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("user@gmail.com", "123").await;

    for i in 0..todo_count {
        let res = client
            .create_todo(Some(&tokens.access_token), Some(&format!("todo{i}")))
            .await;
        assert_eq!(res.status(), StatusCode::CREATED);
    }

    let res = client
        .get_all_todos(&tokens.access_token, limit, None)
        .await;
    let todos = res.json::<TodosPageResponse>().await.unwrap();
    assert_eq!(todos.items.len(), limit);
    assert!(todos.cursor.is_some());
    todo_items.extend(todos.items);

    let res = client
        .get_all_todos(&tokens.access_token, limit, todos.cursor)
        .await;
    let todos = res.json::<TodosPageResponse>().await.unwrap();
    assert_eq!(todos.items.len(), todo_count - limit);
    assert!(todos.cursor.is_none());
    todo_items.extend(todos.items);
    assert_eq!(todo_items.len(), todo_count);

    let any_lost_todo =
        (0..todo_count).any(|suffix| !todo_items.iter().any(|v| v.text == format!("todo{suffix}")));
    assert!(!any_lost_todo);
}

#[tokio::test]
async fn update_todo() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("user@gmail.com", "123").await;

    let res = client.create_todo(Some(&tokens.access_token), None).await;
    let todo_id = res.json::<CreateTodoResponse>().await.unwrap().0;

    let res = client
        .update_todo(&tokens.access_token, &todo_id, "qwerty", "red")
        .await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.get_todo(&tokens.access_token, &todo_id).await;
    assert_eq!(res.status(), StatusCode::OK);

    let todo = res.json::<Todo>().await.unwrap();
    assert_eq!(todo.text, "qwerty");
    assert_eq!(todo.group, "red");
    assert!(todo.completed);
}

#[tokio::test]
async fn update_nonexistent_todo() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("user@gmail.com", "123").await;

    let res = client
        .update_todo(
            &tokens.access_token,
            &TodoId::new().to_string(),
            "qwerty",
            "red",
        )
        .await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_todo_with_empty_patch() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("user@gmail.com", "123").await;

    let res = client
        .update_todo_with_empty_patch(&tokens.access_token, &TodoId::new().to_string())
        .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_todo() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("user@gmail.com", "123").await;

    let res = client.create_todo(Some(&tokens.access_token), None).await;
    let todo_id = res.json::<CreateTodoResponse>().await.unwrap().0;

    let res = client.get_todo(&tokens.access_token, &todo_id).await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.delete_todo(&tokens.access_token, &todo_id).await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.delete_todo(&tokens.access_token, &todo_id).await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = client.get_todo(&tokens.access_token, &todo_id).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_nonexistent_todo() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("user@gmail.com", "123").await;

    let res = client
        .delete_todo(&tokens.access_token, &TodoId::new().to_string())
        .await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}
