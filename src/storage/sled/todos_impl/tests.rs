use super::*;

use crate::{
    storage::sled::test_util::{TestStorageBuilder, ADMIN_UUID},
    Settings,
};

#[tokio::test]
async fn test_get_and_put() {
    let builder = TestStorageBuilder::new();
    let storage = builder.build_todo().await;

    let text = "aaa".to_string();
    let todo = Todo::new(TodoId::new(), &text);
    let id = todo.id;
    storage.put(ADMIN_UUID.into(), id, todo).await.unwrap();

    let todo = storage.get(ADMIN_UUID.into(), id).await.unwrap();

    assert_eq!(todo.text, text);

    let result = storage.get(ADMIN_UUID.into(), TodoId::new()).await;
    assert!(matches!(result, Err(StorageError::NotFound)));
}

#[tokio::test]
async fn test_delete() {
    let builder = TestStorageBuilder::new();
    let storage = builder.build_todo().await;

    let text = "aaa".to_string();
    let todo = Todo::new(TodoId::new(), &text);
    let id = todo.id;
    storage.put(ADMIN_UUID.into(), id, todo).await.unwrap();

    let todo = storage.get(ADMIN_UUID.into(), id).await.unwrap();

    assert_eq!(todo.text, text);

    storage.delete(ADMIN_UUID.into(), id).await.unwrap();

    let result = storage.get(ADMIN_UUID.into(), id).await;
    assert!(matches!(result, Err(StorageError::NotFound)));

    let result = storage.delete(ADMIN_UUID.into(), TodoId::new()).await;

    assert!(matches!(result, Err(StorageError::NoContent)));
}

#[tokio::test]
async fn test_update() {
    let builder = TestStorageBuilder::new();
    let storage = builder.build_todo().await;

    let text = "aaa".to_string();
    let new_text = "bbb".to_string();
    let group = "red".to_string();
    let todo = Todo::new(TodoId::new(), &text);
    let id = todo.id;
    storage.put(ADMIN_UUID.into(), id, todo).await.unwrap();

    let todo = storage.get(ADMIN_UUID.into(), id).await.unwrap();

    assert_eq!(todo.text, text);

    storage
        .update(
            ADMIN_UUID.into(),
            id,
            UpdateTodo {
                text: Some(new_text.clone()),
                completed: Some(true),
                group: Some(group.clone()),
            },
        )
        .await
        .unwrap();

    let updated_todo = storage.get(ADMIN_UUID.into(), id).await.unwrap();
    assert_eq!(updated_todo.text, new_text);
    assert_eq!(updated_todo.group, group);
    assert_eq!(updated_todo.completed, true);

    let result = storage
        .update(
            ADMIN_UUID.into(),
            TodoId::new(),
            UpdateTodo {
                text: None,
                completed: None,
                group: None,
            },
        )
        .await;
    assert!(matches!(result, Err(StorageError::NotFound)));
}

#[tokio::test]
async fn test_get_all_first_page() {
    let todos_count = 15;
    let builder = TestStorageBuilder::new().with_todos(todos_count);
    let storage = builder.build_todo().await;
    let limit = 10;

    let (todos, cursor) = storage
        .get_all(ADMIN_UUID.into(), Pagination { after: None, limit })
        .await
        .unwrap();

    assert_eq!(todos.len(), limit);
    assert!(cursor.is_some());
}

#[tokio::test]
async fn test_get_all_last_page() {
    let todos_count = 15;
    let builder = TestStorageBuilder::new().with_todos(todos_count);
    let mut todos = builder.todos();
    let storage = builder.build_todo().await;
    let limit = 10;

    todos.sort_by_key(|t| t.id);

    let (_, after) = storage
        .get_all(ADMIN_UUID.into(), Pagination { after: None, limit })
        .await
        .unwrap();
    let (todos, after) = storage
        .get_all(ADMIN_UUID.into(), Pagination { after, limit })
        .await
        .unwrap();

    assert_eq!(todos.len(), todos_count - limit);
    assert!(after.is_none());
}

#[tokio::test]
async fn test_get_all_only_one_page() {
    let limit = 10;
    let builder = TestStorageBuilder::new().with_todos(limit);
    let storage = builder.build_todo().await;

    let (todos, after) = storage
        .get_all(ADMIN_UUID.into(), Pagination { after: None, limit })
        .await
        .unwrap();

    assert_eq!(todos.len(), limit);
    assert!(after.is_none());
}

#[tokio::test]
async fn test_get_all_with_full_roundtrip() {
    let limit = 10;
    let todos_count = 25;
    let builder = TestStorageBuilder::new().with_todos(todos_count);
    let mut expected_todos = builder.todos();
    let storage = builder.build_todo().await;
    let mut actual_todos: Vec<Todo> = Vec::with_capacity(todos_count);

    expected_todos.sort_by_key(|t| t.id);

    let (todos, after) = storage
        .get_all(ADMIN_UUID.into(), Pagination { after: None, limit })
        .await
        .unwrap();
    actual_todos.extend(todos);
    let (todos, after) = storage
        .get_all(ADMIN_UUID.into(), Pagination { after, limit })
        .await
        .unwrap();
    actual_todos.extend(todos);
    let (todos, after) = storage
        .get_all(ADMIN_UUID.into(), Pagination { after, limit })
        .await
        .unwrap();
    actual_todos.extend(todos);

    assert_eq!(actual_todos.len(), todos_count);
    assert_eq!(actual_todos, expected_todos);
    assert!(after.is_none());
}

#[tokio::test]
async fn test_get_all_incorrect_cursor() {
    let limit = 10;
    let todos_count = 25;
    let builder = TestStorageBuilder::new().with_todos(todos_count);
    let storage = builder.build_todo().await;

    let (_, _) = storage
        .get_all(ADMIN_UUID.into(), Pagination { after: None, limit })
        .await
        .unwrap();
    let result = storage
        .get_all(
            ADMIN_UUID.into(),
            Pagination {
                after: Some(TodoId::new()),
                limit,
            },
        )
        .await;

    assert!(matches!(result, Err(StorageError::NotFound)));
}

#[tokio::test]
async fn test_get_all_with_deleted_cursor() {
    let limit = 10;
    let todos_count = 25;
    let builder = TestStorageBuilder::new().with_todos(todos_count);
    let storage = builder.build_todo().await;

    let (_, after) = storage
        .get_all(ADMIN_UUID.into(), Pagination { after: None, limit })
        .await
        .unwrap();
    let _ = storage.delete(ADMIN_UUID.into(), after.unwrap()).await;

    let result = storage
        .get_all(ADMIN_UUID.into(), Pagination { after, limit })
        .await;

    assert!(matches!(result, Err(StorageError::NotFound)));
}

#[tokio::test]
async fn test_delete_all_2_pages() {
    let settings = Settings::from_file("test").unwrap();
    let todos_count = settings.storage.sled.unwrap().delete_batch_size * 2;
    let limit = 10;
    let builder = TestStorageBuilder::new().with_todos(todos_count);
    let storage = builder.build_todo().await;

    storage.delete_all(ADMIN_UUID.into()).await.unwrap();

    let (items, next) = storage
        .get_all(ADMIN_UUID.into(), Pagination { after: None, limit })
        .await
        .unwrap();

    assert_eq!(next, None);
    assert_eq!(items.len(), 0);
}

#[tokio::test]
async fn test_delete_all_with_exact_page() {
    let settings = Settings::from_file("test").unwrap();
    let todos_count = settings.storage.sled.unwrap().delete_batch_size;
    let limit = 10;
    let builder = TestStorageBuilder::new().with_todos(todos_count);
    let storage = builder.build_todo().await;

    storage.delete_all(ADMIN_UUID.into()).await.unwrap();

    let (items, next) = storage
        .get_all(ADMIN_UUID.into(), Pagination { after: None, limit })
        .await
        .unwrap();

    assert_eq!(next, None);
    assert_eq!(items.len(), 0);
}

#[tokio::test]
async fn test_delete_all_more_than_2_pages() {
    let settings = Settings::from_file("test").unwrap();
    let todos_count = settings.storage.sled.unwrap().delete_batch_size * 2 + 1;
    let limit = 10;
    let builder = TestStorageBuilder::new().with_todos(todos_count);
    let mut todos = builder.todos();
    todos.sort_by_key(|t| t.id);
    let storage = builder.build_todo().await;

    storage.delete_all(ADMIN_UUID.into()).await.unwrap();

    let (items, next) = storage
        .get_all(ADMIN_UUID.into(), Pagination { after: None, limit })
        .await
        .unwrap();

    assert_eq!(next, None);
    assert_eq!(items.len(), 0);
}
