use super::error::AppError;
use super::types::*;
use crate::{
    handlers::Service,
    storage::{Session, Todo, TodoId, User},
    utils::RootSpan,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use tracing::info;

#[utoipa::path(
    get,
    path = "/todos",
    params(
        ("after" = Option<Uuid>, Query, description = "Cursor ID"),
        ("limit" = usize, Query, description = "Page size")
    ),
    responses(
        (status = 200, description = "List all todos", body = TodosPageResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Unprocessable Entity"),
    ),
    security(("BearerAuth" = [])),
    tag = "todos"
)]
#[tracing::instrument(name = "handlers::todo::get_all", skip_all)]
pub(crate) async fn get_all(
    State(service): State<Service>,
    Extension(root_span): Extension<RootSpan>,
    Extension(user): Extension<User>,
    Extension(session): Extension<Session>,
    params: PaginationParams<TodoId>,
) -> Result<impl IntoResponse, AppError> {
    root_span
        .record()
        .enduser_id(&user.id)
        .session_id(&session.id);

    info!(pagination_params = ?params, "get all todos");

    let (items, cursor) = service.todo().get_all(&user, params.into()).await?;

    info!("Get {} ToDos", items.len());

    Ok(Json(TodosPageResponse { items, cursor }))
}

#[utoipa::path(
    get,
    path = "/todos/{id}",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "ToDo ID")
    ),
    responses(
        (status = 200, description = "Get ToDo by ID", body = Todo),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "ToDo not found"),
        (status = 422, description = "Unprocessable Entity"),
    ),
    tag = "todos"
)]
#[tracing::instrument(name = "handlers::todo::get", skip_all)]
pub(crate) async fn get(
    State(service): State<Service>,
    Extension(root_span): Extension<RootSpan>,
    Extension(user): Extension<User>,
    Extension(session): Extension<Session>,
    Path(id): Path<TodoId>,
) -> Result<impl IntoResponse, AppError> {
    root_span
        .record()
        .enduser_id(&user.id)
        .session_id(&session.id)
        .todo_id(&id);

    let todo = service.todo().get(&user, id).await?;

    tracing::info!(todo = ?todo, "Get ToDo");

    Ok(Json(todo))
}

#[utoipa::path(
    post,
    path = "/todos",
    security(("BearerAuth" = [])),
    request_body(
        content = CreateTodo,
        description = "New ToDo item",
        content_type = "application/json"
    ),
    responses(
        (status = 201, description = "ToDo created", body = String),   // returns ID
        (status = 400, description = "Empty text"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Unprocessable Entity"),
    ),
    tag = "todos"
)]
#[tracing::instrument(name = "handlers::todo::post", skip_all)]
pub(crate) async fn add(
    State(service): State<Service>,
    Extension(root_span): Extension<RootSpan>,
    Extension(user): Extension<User>,
    Extension(session): Extension<Session>,
    Json(input): Json<CreateTodo>,
) -> Result<impl IntoResponse, AppError> {
    root_span
        .record()
        .enduser_id(&user.id)
        .session_id(&session.id);

    match service.todo().add(&user, &input.text).await {
        Ok(id) => {
            root_span.record().todo_id(&id);
            Ok((StatusCode::CREATED, Json(id)))
        }
        Err(e) => {
            tracing::error!(err = ?e, "failed to add new ToDo");
            Err(e)
        }
    }
}

#[utoipa::path(
    patch,
    path = "/todos/{id}",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "ToDo ID")
    ),
    request_body(
        content = UpdateTodo,
        description = "Partial ToDo update",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "ToDo updated"),
        (status = 400, description = "Empty patch"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "ToDo not found"),
        (status = 422, description = "Unprocessable Entity"),
    ),
    tag = "todos"
)]
#[tracing::instrument(name = "handlers::todo::update", skip_all)]
pub(crate) async fn update(
    State(service): State<Service>,
    Extension(root_span): Extension<RootSpan>,
    Extension(user): Extension<User>,
    Extension(session): Extension<Session>,
    Path(id): Path<TodoId>,
    Json(input): Json<UpdateTodo>,
) -> Result<(), AppError> {
    root_span
        .record()
        .enduser_id(&user.id)
        .session_id(&session.id)
        .todo_id(&id);

    if input.completed.is_none() && input.text.is_none() && input.group.is_none() {
        return Err(AppError::EmptyPatch);
    }
    service.todo().update(&user, id, &input).await?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "/todos",
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "All user ToDos deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Unprocessable Entity"),
    ),
    tag = "todos"
)]
#[axum::debug_handler]
#[tracing::instrument(name = "handlers::todo::delete_all", skip_all)]
pub(crate) async fn delete_all(
    State(service): State<Service>,
    Extension(root_span): Extension<RootSpan>,
    Extension(user): Extension<User>,
    Extension(session): Extension<Session>,
) -> Result<(), AppError> {
    root_span
        .record()
        .enduser_id(&user.id)
        .session_id(&session.id);

    service.todo().delete_all(&user).await?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "/todos/{id}",
    security(("BearerAuth" = [])),
    params(
        ("id" = String, Path, description = "ToDo ID")
    ),
    responses(
        (status = 200, description = "ToDo deleted"),
        (status = 204, description = "ToDo not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Unprocessable Entity"),
    ),
    tag = "todos"
)]
#[tracing::instrument(name = "handlers::todo::delete", skip_all)]
pub(crate) async fn delete(
    State(service): State<Service>,
    Extension(root_span): Extension<RootSpan>,
    Extension(user): Extension<User>,
    Extension(session): Extension<Session>,
    Path(id): Path<TodoId>,
) -> Result<(), AppError> {
    root_span
        .record()
        .enduser_id(&user.id)
        .session_id(&session.id)
        .todo_id(&id);

    service.todo().delete(&user, id).await?;

    Ok(())
}
