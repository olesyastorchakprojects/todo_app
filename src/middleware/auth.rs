use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use strum_macros::AsRefStr;
use tracing::{error, info, instrument};

use headers::{authorization::Bearer, Authorization, HeaderMapExt};

use crate::trace_err;
use jsonwebtoken::{decode, errors::ErrorKind, DecodingKey, TokenData, Validation};

use crate::{
    handlers::Service,
    service::jwt::{Claims, TokenKind},
    storage::{Session, User},
    utils::JWT_SECRET_KEY,
};

use super::normalize_uri;

#[derive(Debug, thiserror::Error, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum AuthError {
    #[error("Token has expired")]
    TokenExpired,

    #[error("Session has expired")]
    SessionExpired,

    #[error("Session doesn't exist")]
    InvalidSession,

    #[error("Invalid token")]
    InvalidToken,

    #[error("No such user")]
    InvalidUser,

    #[error("Malformed auth header")]
    InvalidHeader,

    #[error("Expected access token, refresh token provided")]
    ExpectedAccessToken,

    #[error("Expected refresh token, access token provided")]
    ExpectedRefreshToken,

    #[error("Token missing refresh jti")]
    MissingRefreshJti,

    #[error("Session contains invalid refresh jti")]
    InavlidRefreshJti,

    #[error("Failed to decode token")]
    FailedToDecodeToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        // for InvalidSession: {"error": "invalid_session", message: "Session doesn't exist"}
        let body = Json(json!({
            "error": self.as_ref(),
            "message": self.to_string(),
        }));

        (StatusCode::UNAUTHORIZED, body).into_response()
    }
}

#[tracing::instrument(name = "middlewar::auth", skip_all)]
pub(crate) async fn auth(
    State(service): State<Service>,
    headers: axum::http::HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, AuthError> {
    let normalized_uri = normalize_uri(req.uri().path());
    info!(request = ?req, headers = ?headers, uri = normalized_uri, "auth middleware");

    let bearer = trace_err!(validate_header(&headers), "failed to validate header")?;

    let jwt_secret = std::env::var(JWT_SECRET_KEY).map_err(|e| {
        error!(error = ?e, key = JWT_SECRET_KEY, "Failed to load env var");
        AuthError::FailedToDecodeToken
    })?;
    let token_data = trace_err!(
        validate_token(bearer.token(), &jwt_secret),
        "failed to validate token"
    )?;

    trace_err!(
        validate_token_kind(&token_data.claims, &normalized_uri),
        "failed to validate token kind"
    )?;

    let user = trace_err!(
        validate_user(&service, &token_data.claims).await,
        "failed to validate user"
    )?;

    let session = trace_err!(
        validate_session(&service, &token_data.claims).await,
        "failed to validate session"
    )?;
    if token_data.claims.kind == TokenKind::Refresh {
        trace_err!(
            validate_refresh_token(&token_data.claims, &session),
            "failed to validate refresh token"
        )?;
    }

    info!(user_email = %user.email, user_role = ?user.role, session_id = %session.id, "Get user and session from token");

    req.extensions_mut().insert(user);
    req.extensions_mut().insert(session);

    Ok(next.run(req).await)
}

#[instrument(name = "validate_header", skip_all)]
fn validate_header(headers: &axum::http::HeaderMap) -> Result<Authorization<Bearer>, AuthError> {
    headers.typed_get::<Authorization<Bearer>>().ok_or_else(|| {
        error!("Malformed Authorization header");
        AuthError::InvalidHeader
    })
}

#[instrument(name = "validate_token", skip_all)]
fn validate_token(token: &str, jwt_secret: &str) -> Result<TokenData<Claims>, AuthError> {
    let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
    let mut validation = Validation::default();
    validation.leeway = 0;
    decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
        error!(error = ?e, "Error decoding access token");
        match e.into_kind() {
            ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::InvalidToken,
        }
    })
}

#[instrument(name = "validate_token_kind", skip_all)]
fn validate_token_kind(claims: &Claims, normalized_uri: &str) -> Result<(), AuthError> {
    let expected = if normalized_uri.starts_with("/auth/refresh") {
        TokenKind::Refresh
    } else {
        TokenKind::Access
    };
    let actual = &claims.kind;

    match (expected, actual) {
        (TokenKind::Refresh, TokenKind::Access) => Err(AuthError::ExpectedRefreshToken),
        (TokenKind::Access, TokenKind::Refresh) => Err(AuthError::ExpectedAccessToken),
        _ => Ok(()),
    }
}

#[instrument(name = "validate_user", skip_all)]
async fn validate_user(service: &Service, claims: &Claims) -> Result<User, AuthError> {
    service.user().get(claims.sub).await.map_err(|e| {
        error!(error = ?e, "Failed to get user by id from db");
        AuthError::InvalidUser
    })
}

#[instrument(name = "validate_session", skip_all)]
async fn validate_session(service: &Service, claims: &Claims) -> Result<Session, AuthError> {
    let session = service.auth().get(claims.session_id).await.map_err(|e| {
        error!(error = ?e, "Failed to get session by id from db");
        AuthError::InvalidSession
    })?;

    session.validate().map_err(|e| {
        error!(error = ?e, "Session expired");
        AuthError::SessionExpired
    })?;

    Ok(session)
}

#[instrument(name = "validate_session", skip_all)]
fn validate_refresh_token(claims: &Claims, session: &Session) -> Result<(), AuthError> {
    let refresh_jti = claims.refresh_jti.ok_or_else(|| {
        error!("Missing refresh jti in refresh token");
        AuthError::MissingRefreshJti
    })?;

    (session.current_refresh_jti == refresh_jti)
        .then_some(())
        .ok_or(AuthError::InavlidRefreshJti)
}

#[cfg(test)]
mod tests;
