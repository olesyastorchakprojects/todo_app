use std::sync::Arc;

use tracing::{info, instrument};

use crate::{
    handlers::{error::AppError, UpdateTodo},
    storage::{Pagination, Todo, TodoId, TodoStorage, User},
    utils::measure_metrics::measure_and_record_service,
};

pub struct ServiceTodoRef {
    storage: Arc<dyn TodoStorage>,
}

impl ServiceTodoRef {
    pub(crate) fn new(storage: Arc<dyn TodoStorage>) -> Self {
        Self { storage }
    }

    #[instrument(name = "Service::todo::add", skip_all)]
    pub(crate) async fn add(&self, user: &User, text: &str) -> Result<TodoId, AppError> {
        let id = TodoId::new();
        measure_and_record_service("add_todo", || async {
            let todo = Todo::new(id, text);
            self.storage.put(user.id, id, todo).await
        })
        .await?;

        Ok(id)
    }

    #[instrument(name = "Service::todo::get", skip_all)]
    pub(crate) async fn get(&self, user: &User, todo_id: TodoId) -> Result<Todo, AppError> {
        measure_and_record_service("get_todo", || async {
            self.storage.get(user.id, todo_id).await
        })
        .await
        .map_err(Into::into)
    }

    #[instrument(name = "Service::todo::get_all", skip_all, fields(after_is_some = page.after.is_some(),
    limit = page.limit))]
    pub(crate) async fn get_all(
        &self,
        user: &User,
        page: Pagination<TodoId>,
    ) -> Result<(Vec<Todo>, Option<TodoId>), AppError> {
        info!(page_after = ?page.after, "get all todos with page");

        measure_and_record_service("get_all_todos", || async {
            self.storage.get_all(user.id, page).await
        })
        .await
        .map_err(Into::into)
    }

    #[instrument(
        name = "Service::todo::update",
        skip_all,
        fields(text_is_some = patch.text.is_some(),
        group_is_some = patch.group.is_some(),
        completed_is_some = patch.completed.is_some()))
    ]
    pub(crate) async fn update(
        &self,
        user: &User,
        id: TodoId,
        patch: &UpdateTodo,
    ) -> Result<(), AppError> {
        info!(todo_id = %id, "update todo");

        measure_and_record_service("update_todo", || async {
            self.storage.update(user.id, id, patch.into()).await
        })
        .await
        .map_err(Into::into)
    }

    #[instrument(name = "Service::todo::delete_all", skip_all)]
    pub(crate) async fn delete_all(&self, user: &User) -> Result<(), AppError> {
        measure_and_record_service("delete_all_todos", || async {
            self.storage.delete_all(user.id).await
        })
        .await
        .map_err(Into::into)
    }

    #[instrument(name = "Service::todo::delete", skip_all)]
    pub(crate) async fn delete(&self, user: &User, todo_id: TodoId) -> Result<(), AppError> {
        info!(todo_id = %todo_id, "delete todo");

        measure_and_record_service("delete_todo", || async {
            self.storage.delete(user.id, todo_id).await
        })
        .await
        .map_err(Into::into)
    }
}
