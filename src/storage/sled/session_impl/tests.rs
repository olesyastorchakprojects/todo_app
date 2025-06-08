use super::*;

use crate::{
    config::Settings,
    storage::{sled::test_util::TestStorageBuilder, Jti, UserId},
};

#[tokio::test]
async fn test_get_and_put() {
    let builder = TestStorageBuilder::new();
    let storage = builder.build_session().await;
    let settings = Settings::new().unwrap();

    let user_id = UserId::new();
    let refresh_jti = Jti::new();
    let session = Session::new(&user_id, &refresh_jti, &settings.jwt).unwrap();
    storage.put(session.id, session.clone()).await.unwrap();

    let res = storage.get(session.id).await.unwrap();
    assert_eq!(session, res);
}

#[tokio::test]
async fn test_delete() {
    let builder = TestStorageBuilder::new();
    let storage = builder.build_session().await;
    let settings = Settings::new().unwrap();

    let user_id = UserId::new();
    let refresh_jti = Jti::new();
    let session = Session::new(&user_id, &refresh_jti, &settings.jwt).unwrap();
    storage.put(session.id, session.clone()).await.unwrap();

    let res = storage.get(session.id).await.unwrap();
    assert_eq!(session, res);

    storage.delete(session.id).await.unwrap();

    let res = storage.get(session.id).await;
    assert!(matches!(res, Err(StorageError::NotFound)));
}

#[tokio::test]
async fn test_update() {
    let builder = TestStorageBuilder::new();
    let storage = builder.build_session().await;
    let settings = Settings::new().unwrap();

    let user_id = UserId::new();
    let refresh_jti = Jti::new();
    let session = Session::new(&user_id, &refresh_jti, &settings.jwt).unwrap();
    storage.put(session.id, session.clone()).await.unwrap();

    let new_refresh_jti = Jti::new();
    storage.update(session.id, new_refresh_jti).await.unwrap();

    let res = storage.get(session.id).await.unwrap();
    assert_eq!(res.current_refresh_jti, new_refresh_jti);
}
