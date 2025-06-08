mod common;
use common::{create_test_app, spawn_test_app, LoginResponse, TestAppClient};
use reqwest::StatusCode;
use todo_app::{User, UserId, UsersPageResponse};

#[tokio::test]
async fn get_all_users_as_admin() {
    let users_count = 15;
    let limit = 10;
    let mut users = Vec::with_capacity(users_count);
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    for i in 0..users_count {
        let res = client
            .register_user(&format!("user{}@gmail.com", i), "123")
            .await;
        assert_eq!(res.status(), StatusCode::CREATED);
    }

    let res = client.login_user("admin@gmail.com", "admin").await;
    let token = res.json::<LoginResponse>().await.unwrap().access_token;

    let res = client.get_all_users(&token, limit, None).await;

    let users_page = res.json::<UsersPageResponse>().await.unwrap();
    assert_eq!(users_page.items.len(), limit);
    assert!(users_page.cursor.is_some());
    users.extend(users_page.items);

    let res = client.get_all_users(&token, limit, users_page.cursor).await;
    let users_page = res.json::<UsersPageResponse>().await.unwrap();
    assert_eq!(users_page.items.len(), users_count - limit);
    assert!(users_page.cursor.is_none());
    users.extend(users_page.items);

    let any_lost_user = (0..users_count).into_iter().any(|suffix| {
        users
            .iter()
            .find(|v| v.email == format!("user{}@gmail.com", suffix))
            .is_none()
    });
    assert_eq!(any_lost_user, false);
}

#[tokio::test]
async fn get_all_users_as_user() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("user@gmail.com", "123").await;

    let res = client.get_all_users(&tokens.access_token, 10, None).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn promote_user_as_admin() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    client.register_user("user@gmail.com", "123").await;

    let res = client.login_user("admin@gmail.com", "admin").await;
    let token = res.json::<LoginResponse>().await.unwrap().access_token;

    let res = client.get_user_by_email(&token, "user@gmail.com").await;
    let user = res.json::<User>().await.unwrap();

    let res = client.promote_user(&token, &user.id).await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.login_user("user@gmail.com", "123").await;
    let token = res.json::<LoginResponse>().await.unwrap().access_token;

    let res = client.get_all_users(&token, 10, None).await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn promote_user_as_user() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("user@gmail.com", "123").await;

    let res = client
        .promote_user(&tokens.access_token, &UserId::new())
        .await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_user_as_admin() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    client.register_user("user@gmail.com", "123").await;

    let res = client.login_user("admin@gmail.com", "admin").await;
    let token = res.json::<LoginResponse>().await.unwrap().access_token;

    let res = client.get_user_by_email(&token, "user@gmail.com").await;
    let user_by_email = res.json::<User>().await.unwrap();
    assert_eq!(user_by_email.email, "user@gmail.com");

    let res = client.get_user(&token, &user_by_email.id).await;
    let user_by_id = res.json::<User>().await.unwrap();
    assert_eq!(user_by_email, user_by_id);
}

#[tokio::test]
async fn get_user_as_user() {
    let handle = spawn_test_app(create_test_app(None).await).await;

    let client = TestAppClient::new(handle.address);

    let tokens = client.register_and_login("user@gmail.com", "123").await;

    let res = client
        .get_user_by_email(&tokens.access_token, "user@gmail.com")
        .await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    let res = client.get_user(&tokens.access_token, &UserId::new()).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
