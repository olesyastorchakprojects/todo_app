use std::sync::Arc;

use super::*;
use crate::{
    config::{JwtConfig, Settings},
    service::{jwt::generate_access_token, password::create_password_hash, Service},
    storage::{
        test_util::{test_settings, TestStorageBuilder},
        Jti, Role, SessionId, UserId,
    },
};

use chrono::Utc;
use http::HeaderMap;

static EXPIRED_ACCESS_TOKEN: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI2Y2RlMDA1MC1iNTFiLTRiMTMtOGY2Ni00YjcyYTY2ZGQ1NjQiLCJleHAiOjE3NDk1NzI0MDAsInNlc3Npb25faWQiOiIzNTU4MWEwNi05ODllLTQwYmEtOWNmMy02ZWM2OTFjMDVjOGUiLCJyZWZyZXNoX2p0aSI6bnVsbCwia2luZCI6IkFjY2VzcyJ9.i_xEMZ9wb1RdTdei33k-FZ5uPxGqu6sJ4GHW0CNRgdg";

#[test]
fn test_missing_header() {
    let headers = HeaderMap::new();
    let result = validate_header(&headers);

    assert!(matches!(result, Err(AuthError::InvalidHeader)));
}

#[test]
fn test_malformed_header() {
    let mut headers = HeaderMap::new();
    headers.insert(http::header::AUTHORIZATION, "xxx".parse().unwrap());
    let result = validate_header(&headers);

    assert!(matches!(result, Err(AuthError::InvalidHeader)));
}

#[test]
fn test_correct_header() {
    let mut headers = HeaderMap::new();
    headers.insert(http::header::AUTHORIZATION, "Bearer xxx".parse().unwrap());
    let result = validate_header(&headers);

    assert!(result.is_ok());
}

#[test]
fn test_invalid_signature() {
    let settings = Settings::new().unwrap();
    let jwt_secret = std::env::var(JWT_SECRET_KEY).unwrap();
    let token = generate_access_token(
        UserId::new(),
        SessionId::new(),
        &jwt_secret,
        settings.jwt.access_token_ttl_sec,
    )
    .unwrap();
    let result = validate_token(&token, "xxx");
    assert!(matches!(result, Err(AuthError::InvalidToken)));
}

#[test]
fn test_expired_token() {
    let _settings = Settings::new().unwrap();
    let result = validate_token(
        EXPIRED_ACCESS_TOKEN,
        &std::env::var(JWT_SECRET_KEY).unwrap(),
    );
    assert!(matches!(result, Err(AuthError::TokenExpired)));
}

#[test]
fn test_correct_token() {
    let settings = Settings::new().unwrap();
    let jwt_secret = std::env::var(JWT_SECRET_KEY).unwrap();
    let token = generate_access_token(
        UserId::new(),
        SessionId::new(),
        &jwt_secret,
        settings.jwt.access_token_ttl_sec,
    )
    .unwrap();

    let result = validate_token(&token, &jwt_secret);
    assert!(result.is_ok());
}

#[test]
fn test_expected_refresh_token() {
    let claims = &Claims {
        sub: UserId::new(),
        exp: Utc::now().timestamp(),
        session_id: SessionId::new(),
        refresh_jti: None,
        kind: TokenKind::Access,
    };
    let result = validate_token_kind(claims, "/auth/refresh");
    assert!(matches!(result, Err(AuthError::ExpectedRefreshToken)));
}

#[test]
fn test_expected_sccess_token() {
    let claims = &Claims {
        sub: UserId::new(),
        exp: Utc::now().timestamp(),
        session_id: SessionId::new(),
        refresh_jti: None,
        kind: TokenKind::Refresh,
    };
    let result = validate_token_kind(claims, "/todos");
    assert!(matches!(result, Err(AuthError::ExpectedAccessToken)));
}

#[test]
fn test_ok_token_kind_refresh() {
    let claims = &Claims {
        sub: UserId::new(),
        exp: Utc::now().timestamp(),
        session_id: SessionId::new(),
        refresh_jti: None,
        kind: TokenKind::Refresh,
    };
    let result = validate_token_kind(claims, "/auth/refresh");
    assert!(result.is_ok());
}

#[test]
fn test_ok_token_kind_access() {
    let claims = &Claims {
        sub: UserId::new(),
        exp: Utc::now().timestamp(),
        session_id: SessionId::new(),
        refresh_jti: None,
        kind: TokenKind::Access,
    };
    let result = validate_token_kind(claims, "/todos");
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_invalid_user() {
    let test_storage = TestStorageBuilder::new();
    let service = Arc::new(
        Service::new(
            test_storage.build_todo().await,
            test_storage.build_user().await,
            test_storage.build_session().await,
            test_storage.build_flush().await,
        )
        .await,
    );

    let claims = &Claims {
        sub: UserId::new(),
        exp: Utc::now().timestamp(),
        session_id: SessionId::new(),
        refresh_jti: None,
        kind: TokenKind::Access,
    };
    let result = validate_user(&service, claims).await;
    assert!(matches!(result, Err(AuthError::InvalidUser)));
}

#[tokio::test]
async fn test_valid_user() {
    let test_storage = TestStorageBuilder::new();
    let user_storage = test_storage.build_user().await;
    let user_id = UserId::new();
    let user = User {
        id: user_id,
        email: String::new(),
        hashed_password: create_password_hash("password", &test_settings().auth)
            .await
            .unwrap(),
        role: Role::User,
    };
    user_storage.put(user_id, user).await.unwrap();
    let service = Arc::new(
        Service::new(
            test_storage.build_todo().await,
            user_storage,
            test_storage.build_session().await,
            test_storage.build_flush().await,
        )
        .await,
    );

    let claims = &Claims {
        sub: user_id,
        exp: Utc::now().timestamp(),
        session_id: SessionId::new(),
        refresh_jti: None,
        kind: TokenKind::Access,
    };
    let result = validate_user(&service, claims).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_invalid_session() {
    let test_storage = TestStorageBuilder::new();
    let service = Arc::new(
        Service::new(
            test_storage.build_todo().await,
            test_storage.build_user().await,
            test_storage.build_session().await,
            test_storage.build_flush().await,
        )
        .await,
    );

    let claims = &Claims {
        sub: UserId::new(),
        exp: Utc::now().timestamp(),
        session_id: SessionId::new(),
        refresh_jti: None,
        kind: TokenKind::Access,
    };
    let result = validate_session(&service, claims).await;
    assert!(matches!(result, Err(AuthError::InvalidSession)));
}

#[tokio::test]
async fn test_expired_session() {
    let test_storage = TestStorageBuilder::new();
    let session_storage = test_storage.build_session().await;

    let jwt_config = JwtConfig {
        access_token_ttl_sec: 10,
        refresh_token_ttl_sec: 10,
        session_ttl_sec: 0,
    };

    let session = Session::new(&UserId::new(), &Jti::new(), &jwt_config).unwrap();
    let session_id = session.id;
    session_storage.put(session.id, session).await.unwrap();

    let service = Arc::new(
        Service::new(
            test_storage.build_todo().await,
            test_storage.build_user().await,
            session_storage,
            test_storage.build_flush().await,
        )
        .await,
    );

    let claims = &Claims {
        sub: UserId::new(),
        exp: Utc::now().timestamp(),
        session_id,
        refresh_jti: None,
        kind: TokenKind::Access,
    };
    let result = validate_session(&service, claims).await;
    assert!(matches!(result, Err(AuthError::SessionExpired)));
}

#[tokio::test]
async fn test_valid_session() {
    let test_storage = TestStorageBuilder::new();
    let session_storage = test_storage.build_session().await;

    let jwt_config = JwtConfig {
        access_token_ttl_sec: 10,
        refresh_token_ttl_sec: 10,
        session_ttl_sec: 10,
    };

    let session = Session::new(&UserId::new(), &Jti::new(), &jwt_config).unwrap();
    let session_id = session.id;
    session_storage.put(session.id, session).await.unwrap();

    let service = Arc::new(
        Service::new(
            test_storage.build_todo().await,
            test_storage.build_user().await,
            session_storage,
            test_storage.build_flush().await,
        )
        .await,
    );

    let claims = &Claims {
        sub: UserId::new(),
        exp: Utc::now().timestamp(),
        session_id,
        refresh_jti: None,
        kind: TokenKind::Access,
    };
    let result = validate_session(&service, claims).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_refresh_token_jti_none() {
    let settings = Settings::new().unwrap();
    let session = Session::new(&UserId::new(), &Jti::new(), &settings.jwt).unwrap();

    let claims = &Claims {
        sub: UserId::new(),
        exp: Utc::now().timestamp(),
        session_id: session.id,
        refresh_jti: None,
        kind: TokenKind::Access,
    };
    let result = validate_refresh_token(claims, &session);
    assert!(matches!(result, Err(AuthError::MissingRefreshJti)));
}

#[tokio::test]
async fn test_refresh_token_invalid_jti() {
    let settings = Settings::new().unwrap();
    let session = Session::new(&UserId::new(), &Jti::new(), &settings.jwt).unwrap();

    let claims = &Claims {
        sub: UserId::new(),
        exp: Utc::now().timestamp(),
        session_id: session.id,
        refresh_jti: Some(Jti::new()),
        kind: TokenKind::Access,
    };
    let result = validate_refresh_token(claims, &session);
    assert!(matches!(result, Err(AuthError::InavlidRefreshJti)));
}

#[tokio::test]
async fn test_refresh_token_ok() {
    let settings = Settings::new().unwrap();
    let refresh_jti = Jti::new();
    let session = Session::new(&UserId::new(), &refresh_jti, &settings.jwt).unwrap();

    let claims = &Claims {
        sub: UserId::new(),
        exp: Utc::now().timestamp(),
        session_id: session.id,
        refresh_jti: Some(refresh_jti),
        kind: TokenKind::Access,
    };
    let result = validate_refresh_token(claims, &session);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_valid_access_token_flow() {
    let settings = Settings::new().unwrap();
    let user_id = UserId::new();
    let refresh_jti = Jti::new();
    let session = Session::new(&user_id, &refresh_jti, &settings.jwt).unwrap();
    let session_id = session.id;

    let test_storage = TestStorageBuilder::new();
    let session_storage = test_storage.build_session().await;
    let user_storage = test_storage.build_user().await;
    let user = User {
        id: user_id,
        email: String::new(),
        hashed_password: create_password_hash("password", &test_settings().auth)
            .await
            .unwrap(),
        role: Role::User,
    };
    user_storage.put(user_id, user).await.unwrap();
    session_storage.put(session.id, session).await.unwrap();
    let service = Arc::new(
        Service::new(
            test_storage.build_todo().await,
            user_storage,
            session_storage,
            test_storage.build_flush().await,
        )
        .await,
    );

    let token = generate_access_token(
        user_id,
        session_id,
        &std::env::var(JWT_SECRET_KEY).unwrap(),
        settings.jwt.access_token_ttl_sec,
    )
    .unwrap();

    let claims = &Claims {
        sub: user_id,
        exp: Utc::now()
            .checked_add_signed(chrono::Duration::seconds(10))
            .unwrap()
            .timestamp(),
        session_id,
        refresh_jti: None,
        kind: TokenKind::Access,
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {token}").parse().unwrap(),
    );

    let result = validate_header(&headers);
    assert!(result.is_ok());

    let jwt_secret = std::env::var(JWT_SECRET_KEY).unwrap();
    let result = validate_token(&token, &jwt_secret);
    assert!(result.is_ok());

    let result = validate_token_kind(claims, "/todos");
    assert!(result.is_ok());

    let result = validate_user(&service, claims).await;
    assert!(result.is_ok());

    let result = validate_session(&service, claims).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_valid_refresh_token_flow() {
    let settings = Settings::new().unwrap();
    let user_id = UserId::new();
    let refresh_jti = Jti::new();
    let session = Session::new(&user_id, &refresh_jti, &settings.jwt).unwrap();
    let session_id = session.id;

    let test_storage = TestStorageBuilder::new();
    let session_storage = test_storage.build_session().await;
    let user_storage = test_storage.build_user().await;
    let user = User {
        id: user_id,
        email: String::new(),
        hashed_password: create_password_hash("password", &test_settings().auth)
            .await
            .unwrap(),
        role: Role::User,
    };
    user_storage.put(user_id, user).await.unwrap();
    session_storage
        .put(session.id, session.clone())
        .await
        .unwrap();
    let service = Arc::new(
        Service::new(
            test_storage.build_todo().await,
            user_storage,
            session_storage,
            test_storage.build_flush().await,
        )
        .await,
    );

    let token = generate_access_token(
        user_id,
        session_id,
        &std::env::var(JWT_SECRET_KEY).unwrap(),
        settings.jwt.access_token_ttl_sec,
    )
    .unwrap();

    let claims = &Claims {
        sub: user_id,
        exp: Utc::now()
            .checked_add_signed(chrono::Duration::seconds(10))
            .unwrap()
            .timestamp(),
        session_id,
        refresh_jti: Some(refresh_jti),
        kind: TokenKind::Refresh,
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {token}").parse().unwrap(),
    );

    let result = validate_header(&headers);
    assert!(result.is_ok());

    let jwt_secret = std::env::var(JWT_SECRET_KEY).unwrap();
    let result = validate_token(&token, &jwt_secret);
    assert!(result.is_ok());

    let result = validate_token_kind(claims, "/auth/refresh");
    assert!(result.is_ok());

    let result = validate_user(&service, claims).await;
    assert!(result.is_ok());

    let result = validate_session(&service, claims).await;
    assert!(result.is_ok());

    let result = validate_refresh_token(claims, &session);
    assert!(result.is_ok());
}
