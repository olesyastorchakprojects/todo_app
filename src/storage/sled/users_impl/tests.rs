use super::*;

use crate::{
    service::password::create_password_hash,
    storage::{
        sled::test_util::{TestStorageBuilder, ADMIN_UUID},
        test_util::test_settings,
    },
    Settings,
};

#[tokio::test]
async fn test_get_users_first_page() {
    let user_count = 15;
    let builder = TestStorageBuilder::new().with_users(user_count).await;
    let storage = builder.build_user().await;
    let limit = 10;

    let (users, cursor) = storage
        .get_all(ADMIN_UUID.into(), Pagination { after: None, limit })
        .await
        .unwrap();

    assert_eq!(users.len(), limit);
    assert!(cursor.is_some());
}

#[tokio::test]
async fn test_get_users_last_page() {
    let user_count = 15;
    let builder = TestStorageBuilder::new().with_users(user_count).await;
    let storage = builder.build_user().await;
    let limit = 10;

    let (_, cursor) = storage
        .get_all(ADMIN_UUID.into(), Pagination { after: None, limit })
        .await
        .unwrap();

    let (users, cursor) = storage
        .get_all(
            ADMIN_UUID.into(),
            Pagination {
                after: cursor,
                limit,
            },
        )
        .await
        .unwrap();

    assert_eq!(users.len(), user_count - limit);
    assert!(cursor.is_none());
}

#[tokio::test]
async fn test_get_user_with_id() {
    let storage = TestStorageBuilder::new().build_user().await;

    let user_id = UserId::new();
    let email = "aaa@gmail.com".to_string();
    let new_user = User {
        id: user_id.into(),
        email,
        hashed_password: create_password_hash("password", &test_settings().auth)
            .await
            .unwrap(),
        role: Role::User,
    };

    let result = storage.put(user_id, new_user.clone()).await;
    assert!(result.is_ok());
    let user = storage.get(user_id).await.unwrap();
    assert_eq!(user, new_user);
    let result = storage.get(UserId::new()).await;
    assert!(matches!(result, Err(StorageError::NotFound)));
}

#[tokio::test]
async fn test_get_user_with_email() {
    let storage = TestStorageBuilder::new().build_user().await;

    let user_id = UserId::new();
    let email = "aaa1@gmail.com".to_string();
    let new_user = User {
        id: user_id.into(),
        email: email.clone(),
        hashed_password: create_password_hash("password", &test_settings().auth)
            .await
            .unwrap(),
        role: Role::User,
    };

    let result = storage.put(user_id, new_user.clone()).await;
    assert!(result.is_ok());
    let user = storage.get_by_email(&email).await.unwrap();
    assert_eq!(user, new_user);
    let result = storage.get_by_email("xxx").await;
    assert!(matches!(result, Err(StorageError::NotFound)));
}

#[tokio::test]
async fn test_update_user_role() {
    let storage = TestStorageBuilder::new().build_user().await;

    let user_id = UserId::new();
    let email = "aaa1@gmail.com".to_string();
    let new_user = User {
        id: user_id.into(),
        email: email.clone(),
        hashed_password: create_password_hash("password", &test_settings().auth)
            .await
            .unwrap(),
        role: Role::User,
    };
    storage.put(user_id, new_user.clone()).await.unwrap();

    storage.update_role(user_id, Role::Admin).await.unwrap();

    let user = storage.get(user_id).await.unwrap();
    assert_eq!(user.role, Role::Admin);

    let user = storage.get_by_email(&email).await.unwrap();
    assert_eq!(user.role, Role::Admin);

    let result = storage.update_role(UserId::new(), Role::Admin).await;
    assert!(matches!(result, Err(StorageError::NotFound)));
}

#[tokio::test]
async fn test_delete_user() {
    let settings = Settings::from_file("test").unwrap();
    let todos_count = settings.storage.sled.unwrap().delete_batch_size;
    let limit = 20;
    let builder = TestStorageBuilder::new().with_todos(todos_count);
    let todo_storage = builder.build_todo().await;
    let user_storage = builder.build_user().await;

    let email = "aaa@gmail.com".to_string();
    let new_user = User {
        id: ADMIN_UUID.into(),
        email: email.clone(),
        hashed_password: create_password_hash("password", &test_settings().auth)
            .await
            .unwrap(),
        role: Role::User,
    };
    user_storage.put(ADMIN_UUID.into(), new_user).await.unwrap();

    let (items, next) = todo_storage
        .get_all(ADMIN_UUID.into(), Pagination { after: None, limit })
        .await
        .unwrap();

    assert_eq!(next, None);
    assert_eq!(items.len(), todos_count);

    user_storage.delete(ADMIN_UUID.into()).await.unwrap();

    let result = user_storage.get(ADMIN_UUID.into()).await;
    assert!(matches!(result, Err(StorageError::NotFound)));
    let result = user_storage.get_by_email(&email).await;
    assert!(matches!(result, Err(StorageError::NotFound)));

    let (items, next) = todo_storage
        .get_all(ADMIN_UUID.into(), Pagination { after: None, limit })
        .await
        .unwrap();

    assert_eq!(next, None);
    assert_eq!(items.len(), 0);
}
