use crate::config::types::SledConfig;
use crate::storage::sled::internal::TreeScan;
use crate::storage::{TodoId, UserId};
use crate::trace_err;
use crate::utils::blocking_task_guard::BlockingTaskGuard;
use crate::utils::measure_metrics::measure_and_record_storage;

use super::error::SledStorageError;
use super::internal::{
    span_wrappers::{
        deserialize_in_span, deserialize_in_transaction_with_span,
        get_value_in_transaction_with_span, get_value_with_span,
        insert_value_in_transaction_with_span, insert_value_with_span,
        remove_batch_in_transaction_with_span, remove_value_with_span, serialize_in_span,
        serialize_in_transaction_with_span,
    },
    Key, KeyPrefix, PrefixKind,
};
use super::{todo_key, FromBytesWithConfig};
use super::{BincodeConfig, SledStorage};
use super::{Pagination, StorageError, Todo, TodoStorage, TodoVersion, UpdateTodo};
use async_trait::async_trait;
use sled::Tree;
use tracing::{debug, info, info_span, instrument, Span};

#[async_trait]
impl TodoStorage for SledStorage {
    #[instrument(name = "SledStorage::get_todo", skip_all)]
    async fn get(&self, user_id: UserId, todo_id: TodoId) -> Result<Todo, StorageError> {
        info!(user_id = %user_id, todo_id = %todo_id, "get todo");

        measure_and_record_storage("SledStorage::get_todo", || {
            let key = todo_key(&user_id, &todo_id);

            let value = trace_err!(
                get_value_with_span(&key, &self.todo_tree),
                "failed to read todo from storage"
            )?;

            Ok::<Todo, SledStorageError>(
                trace_err!(
                    deserialize_in_span::<TodoVersion>(&self.bincode_config, &value),
                    "failed to bin decode todo"
                )?
                .into(),
            )
        })
        .map_err(Into::into)
    }

    #[instrument(name = "SledStorage::put_todo", skip_all)]
    async fn put(&self, user_id: UserId, todo_id: TodoId, item: Todo) -> Result<(), StorageError> {
        info!(user_id = %user_id, todo_id = %todo_id, "put todo");

        measure_and_record_storage("SledStorage::put_todo", || {
            let key = todo_key(&user_id, &todo_id);

            let todo_version: TodoVersion = item.into();

            let encoded: Vec<u8> = trace_err!(
                serialize_in_span(&self.bincode_config, &todo_version),
                "failed to bin decode todo"
            )?;

            trace_err!(
                insert_value_with_span(&key, &encoded, &self.todo_tree),
                "failed to write todo into storage"
            )
        })
        .map_err(Into::into)
    }

    #[instrument(name = "SledStorage::delete_todo", skip_all)]
    async fn delete(&self, user_id: UserId, todo_id: TodoId) -> Result<(), StorageError> {
        info!(user_id = %user_id, todo_id = %todo_id, "delete todo");

        measure_and_record_storage("SledStorage::delete_todo", || {
            let key = todo_key(&user_id, &todo_id);

            trace_err!(
                remove_value_with_span(&key, &self.todo_tree),
                "failed to remove todo from storage"
            )
        })
        .map_err(Into::into)
    }

    #[instrument(name = "SledStorage::update_todo", skip_all)]
    async fn update(
        &self,
        user_id: UserId,
        todo_id: TodoId,
        patch: UpdateTodo,
    ) -> Result<(), StorageError> {
        // cloning tree should be cheap: struct Tree{inner: Arc<TreeInner>}
        let (todo_tree, bincode_config) = info_span!("Cloning trees and config")
            .in_scope(|| (self.todo_tree.clone(), self.bincode_config));

        let span = Span::current();
        tokio::task::spawn_blocking(move || {
            let _guard = BlockingTaskGuard::new("update_todo");
            span.in_scope(|| update_todo(user_id, todo_id, patch, &todo_tree, &bincode_config))
        })
        .await?
    }

    #[instrument(name = "SledStorage::storage::get_all", skip_all)]
    async fn get_all(
        &self,
        user_id: UserId,
        pagination: Pagination<TodoId>,
    ) -> Result<(Vec<Todo>, Option<TodoId>), StorageError> {
        info!(user_id = %user_id, pagination = ?pagination, "get all todo");

        let result: Result<_, SledStorageError> =
            measure_and_record_storage("SledStorage::get_all", || {
                let after_key = match pagination.after {
                    Some(todo_id) => todo_key(&user_id, &todo_id),
                    None => Key::new(KeyPrefix::from_kind(PrefixKind::Todo), user_id),
                };

                let page = info_span!("TreeScan::scan_from::within::until_pagination::collect")
                    .in_scope(|| {
                        trace_err!(
                            TreeScan::scan_from(&self.todo_tree, &after_key)
                                .within(KeyPrefix::new(PrefixKind::Todo, user_id))
                                .with_pagination(pagination)
                                .collect(
                                    &self.bincode_config,
                                    |_, bytes, config| {
                                        Ok(Todo::from(TodoVersion::from_bytes(bytes, config)?))
                                    },
                                    None,
                                ),
                            "failed to do tree scan to get page of todo-s"
                        )
                    })?;
                Ok((page.items, page.next_cursor))
            });

        Ok(result?)
    }

    #[instrument(name = "SledStorage::delete_all_todos", skip_all)]
    async fn delete_all(&self, user_id: UserId) -> Result<(), StorageError> {
        // cloning tree should be cheap: struct Tree{inner: Arc<TreeInner>}
        let (todo_tree, bincode_config, settings) = info_span!("Cloning trees and config")
            .in_scope(|| {
                (
                    self.todo_tree.clone(),
                    self.bincode_config,
                    self.storage_settings.clone(),
                )
            });

        let span = Span::current();
        tokio::task::spawn_blocking(move || {
            let _guard = BlockingTaskGuard::new("delete_all_todos");
            span.in_scope(|| delete_all_todos(user_id, &todo_tree, &bincode_config, &settings))
        })
        .await?
    }
}

#[instrument(name = "SledStorage::delete_all_todos", skip_all)]
fn delete_all_todos(
    user_id: UserId,
    todo_tree: &Tree,
    bincode_config: &BincodeConfig,
    settings: &SledConfig,
) -> Result<(), StorageError> {
    info!(user_id = %user_id, "delete all todo");

    let result: Result<_, SledStorageError> = measure_and_record_storage(
        "SledStorage::delete_all_user_todos",
        || {
            let first_key = Key::new(KeyPrefix::from_kind(PrefixKind::Todo), user_id);
            let key_prefix = KeyPrefix::new(PrefixKind::Todo, user_id);
            let mut after: Option<Key> = None;
            let mut deleted_items = 0;
            loop {
                let after_key = after.as_ref().unwrap_or(&first_key);

                info!(after_key = %after_key, key_prefix = %key_prefix, "TreeScan input");
                let mut page = trace_err!(
                    TreeScan::scan_from(todo_tree, after_key)
                        .within(key_prefix.clone())
                        .with_pagination(Pagination {
                            after: after.clone(),
                            limit: settings.delete_batch_size,
                        })
                        .collect(bincode_config, |key, _, _| Ok(key.clone()), None),
                    "failed to do tree scan to get page of todo-s to delete"
                )?;

                // Delete cursor from prev iteration
                if let Some(prev_cursor) = after {
                    debug!("insert prev cursor into page.items");
                    page.items.insert(0, prev_cursor);
                }
                // Keep cursor for next iteration
                if page.next_cursor.is_some() {
                    debug!("page.next_cursor is some. pop cursor for the next iteration from page.items");
                    page.items.pop();
                }

                todo_tree.transaction(|tx| {
                    trace_err!(
                        remove_batch_in_transaction_with_span(&page.items, tx),
                        "Failed to remove batch of todo-s"
                    )?;
                    Ok(())
                })?;

                deleted_items += page.items.len();
                match page.next_cursor {
                    Some(cursor) => after = Some(cursor),
                    None => break,
                }
            }
            info!(count = deleted_items, "deleted todos");
            Ok(())
        },
    );
    Ok(result?)
}

#[instrument(name = "update_todo", skip_all)]
fn update_todo(
    user_id: UserId,
    todo_id: TodoId,
    patch: UpdateTodo,
    todo_tree: &Tree,
    bincode_config: &BincodeConfig,
) -> Result<(), StorageError> {
    info!(user_id = %user_id, todo_id = %todo_id, "update todo");

    measure_and_record_storage("SledStorage::update_todo_in_transaction", || {
        todo_tree.transaction(|tx| {
            let key = todo_key(&user_id, &todo_id);
            let value = trace_err!(
                get_value_in_transaction_with_span(&key, tx),
                "failed to read todo from storage"
            )?;

            if let Some(value) = value {
                let mut todo: Todo = trace_err!(
                    deserialize_in_transaction_with_span::<TodoVersion>(bincode_config, &value,),
                    "failed to bin decode todo"
                )?
                .into();

                todo.apply(&patch);

                let encoded = trace_err!(
                    serialize_in_transaction_with_span(bincode_config, &TodoVersion::from(todo),),
                    "failed to bin encode todo"
                )?;

                trace_err!(
                    insert_value_in_transaction_with_span(&key, &encoded, tx),
                    "failed to write todo into storage"
                )?;

                Ok(())
            } else {
                tracing::error!("failed to find todo in the storage");
                Err(SledStorageError::NotFound.into())
            }
        })
    })
    .map_err(SledStorageError::from)?;

    Ok(())
}

#[cfg(test)]
mod tests;
