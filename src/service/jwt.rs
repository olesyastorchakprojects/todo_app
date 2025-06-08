use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use tracing::{info, info_span, instrument};

use crate::{
    service::AppError,
    storage::{Jti, SessionId, UserId},
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TokenKind {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: UserId,
    pub exp: usize, // expiration token time
    pub session_id: SessionId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: UserId,
    pub exp: i64,
    pub session_id: SessionId,
    pub refresh_jti: Option<Jti>,
    pub kind: TokenKind,
}

#[instrument(name = "generate_access_token", skip_all)]
pub fn generate_access_token(
    user_id: UserId,
    session_id: SessionId,
    jwt_encoding_key: &str,
    ttl_sec: i64,
) -> Result<String, AppError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(ttl_sec))
        .ok_or(AppError::InvalidTtl)?
        .timestamp();

    let claims = Claims {
        sub: user_id,
        exp: expiration,
        session_id,
        refresh_jti: None,
        kind: TokenKind::Access,
    };

    info_span!("encode_jwt_token").in_scope(|| {
        info!(user_id = %user_id, "encoding access jwt token");
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_encoding_key.as_bytes()),
        )
        .map_err(Into::into)
    })
}

#[instrument(name = "generate_access_token", skip_all)]
pub fn generate_refresh_token(
    user_id: UserId,
    session_id: SessionId,
    refresh_jti: Jti,
    jwt_encoding_key: &str,
    ttl_secs: i64,
) -> Result<String, AppError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(ttl_secs))
        .ok_or(AppError::InvalidTtl)?
        .timestamp();

    let claims = Claims {
        sub: user_id,
        exp: expiration,
        session_id,
        refresh_jti: Some(refresh_jti),
        kind: TokenKind::Refresh,
    };

    info_span!("encode_jwt_token").in_scope(|| {
        info!(user_id = %user_id, "encoding refresh jwt token");
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_encoding_key.as_bytes()),
        )
        .map_err(Into::into)
    })
}
