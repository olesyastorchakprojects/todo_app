use super::error::AppError;
use super::types::*;
use super::Service;
use crate::storage::{Role, Session, User, UserId};
use crate::utils::RootSpan;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};
use std::str::FromStr;

#[utoipa::path(
    get,
    path = "/admin/users",
    params(
        ("after" = Option<Uuid>, Query, description = "Cursor ID"),
        ("limit" = usize, Query, description = "Page size")
    ),
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "List all users", body = UsersPageResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Unprocessable Entity"),
    ),
    tag = "admin"
)]
#[tracing::instrument(name = "handlers::admin::get_all", skip_all)]
pub(crate) async fn get_all(
    State(service): State<Service>,
    Extension(root_span): Extension<RootSpan>,
    Extension(user): Extension<User>,
    Extension(session): Extension<Session>,
    params: PaginationParams<UserId>,
) -> Result<impl IntoResponse, AppError> {
    root_span
        .record()
        .enduser_id(&user.id)
        .session_id(&session.id);

    let result = service.user().get_all(&user, params.into()).await;
    let (items, cursor) = result?;

    tracing::info!(count = items.len(), "Get users");

    Ok(Json(UsersPageResponse { items, cursor }))
}

#[utoipa::path(
    patch,
    path = "/admin/user/{id}/role",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "User ID")
    ),
    request_body(
        content = UpdateRole,
        description = "New user role",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Role updated"),
        (status = 400, description = "Invalid role"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "User not found"),
        (status = 422, description = "Unprocessable Entity"),
    ),
    tag = "admin"
)]
#[tracing::instrument(name = "handlers::admin::update", skip_all)]
pub(crate) async fn update(
    State(service): State<Service>,
    Extension(root_span): Extension<RootSpan>,
    Extension(user): Extension<User>,
    Extension(session): Extension<Session>,
    Path(user_id): Path<UserId>,
    Json(input): Json<UpdateRole>,
) -> Result<(), AppError> {
    root_span
        .record()
        .enduser_id(&user.id)
        .session_id(&session.id)
        .target_user_id(&user_id);

    service
        .user()
        .update(
            &user,
            user_id,
            Role::from_str(&input.role).map_err(|_| AppError::InvalidRole(input.role))?,
        )
        .await?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "/admin/user/{id}",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User deleted"),
        (status = 204, description = "User not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Unprocessable Entity"),
    ),
    tag = "admin"
)]
#[tracing::instrument(name = "handlers::admin::delete", skip_all)]
pub(crate) async fn delete(
    State(service): State<Service>,
    Extension(root_span): Extension<RootSpan>,
    Extension(user): Extension<User>,
    Extension(session): Extension<Session>,
    Path(user_id): Path<UserId>,
) -> Result<(), AppError> {
    root_span
        .record()
        .enduser_id(&user.id)
        .session_id(&session.id)
        .target_user_id(&user_id);

    service.user().delete(&user, user_id).await?;

    Ok(())
}

#[utoipa::path(
    get,
    path = "/admin/user/{id}",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Get user by ID", body = DisplayUser),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "User not found"),
        (status = 422, description = "Unprocessable Entity"),
    ),
    tag = "admin"
)]
#[tracing::instrument(name = "handlers::admin::get", skip_all)]
pub(crate) async fn get(
    State(service): State<Service>,
    Extension(root_span): Extension<RootSpan>,
    Extension(user): Extension<User>,
    Extension(session): Extension<Session>,
    Path(user_id): Path<UserId>,
) -> Result<impl IntoResponse, AppError> {
    root_span
        .record()
        .enduser_id(&user.id)
        .session_id(&session.id)
        .target_user_id(&user_id);

    let user = service.user().get(user_id).await?;

    Ok(Json(user))
}

#[utoipa::path(
    get,
    path = "/admin/user/email/{email}",
    security(("BearerAuth" = [])),
    params(
        ("email" = String, Path, description = "User email")
    ),
    responses(
        (status = 200, description = "Get user by email", body = DisplayUser),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "User not found"),
        (status = 422, description = "Unprocessable Entity"),
    ),
    tag = "admin"
)]
#[tracing::instrument(name = "handlers::user::get_by_email", skip_all)]
pub(crate) async fn get_by_email(
    State(service): State<Service>,
    Extension(root_span): Extension<RootSpan>,
    Extension(user): Extension<User>,
    Extension(session): Extension<Session>,
    Path(email): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    root_span
        .record()
        .enduser_id(&user.id)
        .session_id(&session.id)
        .target_user_email(&email);

    let user = service.user().get_by_email(&email).await?;

    Ok(Json(user))
}
