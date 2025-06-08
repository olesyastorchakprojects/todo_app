use std::str::FromStr;

use axum::{
    extract::{FromRequestParts, Query},
    http::{request::Parts, StatusCode},
};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};
use utoipa::ToSchema;

use crate::storage::{Role, StorageError, Todo, TodoId, User, UserId};

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct RegisterUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct LoginToken {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateTodo {
    pub text: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateTodo {
    pub text: Option<String>,
    pub completed: Option<bool>,
    pub group: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateRole {
    pub role: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum PaginationParams<Id> {
    NextPage { after: Id, limit: usize },
    FirstPage { limit: usize },
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct TodosPageResponse {
    pub items: Vec<Todo>,
    #[schema(value_type = String)]
    pub cursor: Option<TodoId>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ToSchema)]
pub struct DisplayUser {
    #[schema(value_type = String)]
    pub id: UserId,
    pub email: String,
    #[schema(value_type = String)]
    pub role: Role,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct UsersPageResponse {
    pub items: Vec<DisplayUser>,
    #[schema(value_type = String)]
    pub cursor: Option<UserId>,
}

#[derive(Debug, Deserialize)]
struct RawPagination<Id> {
    after: Option<Id>,
    limit: Option<usize>,
}

impl<S, Id> FromRequestParts<S> for PaginationParams<Id>
where
    S: Send + Sync,
    Id: std::fmt::Debug + FromStr<Err = StorageError> + for<'de> Deserialize<'de>,
{
    type Rejection = (StatusCode, &'static str);

    #[instrument(name = "construct_pagination_params_from_parts", skip_all)]
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Query(raw): Query<RawPagination<Id>> = Query::from_request_parts(parts, _state)
            .await
            .map_err(|e| {
            error!(error = ?e, "Failed to construct RawPagination from request parts");
            (axum::http::StatusCode::BAD_REQUEST, "Invalid input")
        })?;

        let after_is_some = raw.after.is_some();
        match (raw.after, raw.limit) {
            (Some(after), Some(limit)) => Ok(PaginationParams::NextPage { after, limit }),
            (None, Some(limit)) => Ok(PaginationParams::FirstPage { limit }),
            _ => {
                error!(
                    after_is_some,
                    limit_is_some = raw.limit.is_some(),
                    "Invalid pagination input: expect `limit` and optional `after`"
                );
                Err((
                    axum::http::StatusCode::BAD_REQUEST,
                    "Must provide `limit`, and optionally `after`.",
                ))
            }
        }
    }
}

impl From<User> for DisplayUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            role: user.role,
        }
    }
}
