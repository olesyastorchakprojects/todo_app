use super::error::AppError;
use super::{LoginToken, LoginUser, RegisterUser, Service};
use crate::config::Settings;
use crate::storage::{Session, User};
use crate::utils::RootSpan;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};

#[utoipa::path(
    post,
    path = "/auth/logout",
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "User logged out successfully"),
        (status = 401, description = "Invalid token"),
        (status = 422, description = "Unprocessable Entity"),
    ),
    tag = "auth"
)]
#[tracing::instrument(name = "handlers::auth::logout", skip_all)]
pub(crate) async fn logout(
    State(service): State<Service>,
    Extension(root_span): Extension<RootSpan>,
    Extension(session): Extension<Session>,
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, AppError> {
    root_span
        .record()
        .enduser_id(&user.id)
        .session_id(&session.id);

    match service.auth().delete(session.id).await {
        Err(e) => Err(e),
        Ok(()) => Ok(StatusCode::NO_CONTENT.into_response()),
    }
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Tokens generated successfully", body = LoginToken),
        (status = 401, description = "Invalid token"),
        (status = 422, description = "Unprocessable Entity"),
    ),
    tag = "auth"
)]
#[tracing::instrument(name = "handlers::auth::refresh_token", skip_all)]
pub(crate) async fn refresh(
    State(service): State<Service>,
    Extension(root_span): Extension<RootSpan>,
    Extension(user): Extension<User>,
    Extension(session): Extension<Session>,
    Extension(settings): Extension<Settings>,
) -> Result<impl IntoResponse, AppError> {
    root_span
        .record()
        .enduser_id(&user.id)
        .session_id(&session.id);

    let tokens = service
        .auth()
        .refresh_token(&session, &settings.jwt)
        .await?;

    Ok(Json(tokens))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body(
        content = RegisterUser,
        description = "Register user input",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "User logged in successfully", body = LoginToken),
        (status = 400, description = "Missing email or password"),
        (status = 401, description = "Wrong password"),
        (status = 422, description = "Unprocessable Entity"),
    ),
    tag = "auth"
)]
#[tracing::instrument(name = "handlers::auth::login", skip_all)]
pub(crate) async fn login(
    State(service): State<Service>,
    Extension(settings): Extension<Settings>,
    Extension(root_span): Extension<RootSpan>,
    Json(input): Json<LoginUser>,
) -> Result<impl IntoResponse, AppError> {
    root_span.record().enduser_email(&input.email);

    if input.email.is_empty() || input.password.is_empty() {
        return Err(AppError::MissingPasswordEmail);
    }

    let tokens = service.login_user(input, &settings).await?;

    Ok(Json(tokens))
}

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body(
        content = RegisterUser,
        description = "Register user input",
        content_type = "application/json"
    ),
    responses(
        (status = 201, description = "User created successfully"),
        (status = 400, description = "Missing email or password"),
        (status = 409, description = "User with email already registered"),
        (status = 422, description = "Unprocessable Entity"),
    ),
    tag = "auth"
)]
#[tracing::instrument(name = "handlers::auth::register", skip_all)]
pub(crate) async fn register(
    State(service): State<Service>,
    Extension(root_span): Extension<RootSpan>,
    Extension(settings): Extension<Settings>,
    Json(input): Json<RegisterUser>,
) -> Result<impl IntoResponse, AppError> {
    root_span.record().enduser_email(&input.email);

    if input.email.is_empty() || input.password.is_empty() {
        return Err(AppError::MissingPasswordEmail);
    }
    let result = service.user().add(input, &settings).await;

    match result {
        Ok(_) => Ok((StatusCode::CREATED, "User created")),
        Err(e) => Err(e),
    }
}
