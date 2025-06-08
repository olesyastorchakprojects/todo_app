use super::error::SledStorageError;
use super::SledStorage;
use crate::{
    storage::{
        sled::{
            internal::span_wrappers::flush_tree_in_span, SLED_EMAIL_TREE, SLED_SESSION_TREE,
            SLED_TODO_TREE, SLED_USER_TREE,
        },
        FlushStorage, StorageError,
    },
    trace_err,
    utils::measure_metrics::measure_and_record_storage,
};
use async_trait::async_trait;
use tracing::instrument;

#[async_trait]
impl FlushStorage for SledStorage {
    #[instrument(name = "SledStorage::flush", skip_all)]
    async fn flush(&self) -> Result<(), StorageError> {
        measure_and_record_storage("SledStorage::flush", || {
            trace_err!(
                flush_tree_in_span(&self.user_tree, SLED_USER_TREE),
                "failed to flush user_tree"
            )?;

            trace_err!(
                flush_tree_in_span(&self.email_tree, SLED_EMAIL_TREE),
                "failed to flush email_tree"
            )?;

            trace_err!(
                flush_tree_in_span(&self.session_tree, SLED_SESSION_TREE),
                "failed to flush session_tree"
            )?;

            trace_err!(
                flush_tree_in_span(&self.todo_tree, SLED_TODO_TREE),
                "failed to flush todo_tree"
            )?;

            Ok::<(), SledStorageError>(())
        })
        .map_err(Into::into)
    }
}
