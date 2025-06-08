use crate::trace_err;
use async_trait::async_trait;
use tracing::{info, instrument};

use crate::{
    storage::{
        session::Session,
        sled::{
            error::SledStorageError,
            internal::span_wrappers::{
                deserialize_in_span, deserialize_in_transaction_with_span,
                get_value_in_transaction_with_span, get_value_with_span,
                insert_value_in_transaction_with_span, insert_value_with_span,
                remove_value_with_span, serialize_in_span, serialize_in_transaction_with_span,
            },
            session_key,
        },
        Jti, SessionId, SessionStorage, StorageError,
    },
    utils::measure_metrics::measure_and_record_storage,
};

use super::SledStorage;

#[async_trait]
impl SessionStorage for SledStorage {
    #[instrument(name = "SledStorage::session::get", skip_all)]
    async fn get(&self, id: SessionId) -> Result<Session, StorageError> {
        info!(session_id = %id, "get session");

        measure_and_record_storage("SledStorage::session::get", || {
            let key = session_key(&id);

            let value = trace_err!(
                get_value_with_span(&key, &self.session_tree),
                "failed to read session from storage"
            )?;

            deserialize_in_span(&self.bincode_config, &value)
        })
        .map_err(Into::into)
    }

    #[instrument(name = "SledStorage::session::put", skip_all)]
    async fn put(&self, id: SessionId, session: Session) -> Result<(), StorageError> {
        info!(session_id = %id, user_id = %session.user_id, "put session");

        measure_and_record_storage("SledStorage::put_session", || {
            let key = session_key(&id);

            let encoded: Vec<u8> = trace_err!(
                serialize_in_span(&self.bincode_config, &session),
                "failed to bin encode session"
            )?;

            trace_err!(
                insert_value_with_span(&key, &encoded, &self.session_tree),
                "failed to write session into storage"
            )
        })
        .map_err(Into::into)
    }

    #[instrument(name = "SledStorage::session::delete", skip_all)]
    async fn delete(&self, id: SessionId) -> Result<(), StorageError> {
        info!(session_id = %id, "delete session");

        measure_and_record_storage("SledStorage::delete_todo", || {
            let key = session_key(&id);

            trace_err!(
                remove_value_with_span(&key, &self.session_tree),
                "failed to delete session from storage"
            )
        })
        .map_err(Into::into)
    }

    #[instrument(name = "SledStorage::session::update", skip_all)]
    async fn update(&self, id: SessionId, refresh_jti: Jti) -> Result<(), StorageError> {
        info!(refresh_jti = %refresh_jti, "update session");

        measure_and_record_storage("SledStorage::update_session_in_transaction", || {
            self.session_tree.transaction(|tx| {
                let key = session_key(&id);
                let value = trace_err!(
                    get_value_in_transaction_with_span(&key, tx),
                    "failed to read session from storage"
                )?;

                if let Some(value) = value {
                    let mut session = trace_err!(
                        deserialize_in_transaction_with_span::<Session>(
                            &self.bincode_config,
                            &value,
                        ),
                        "failed to bin decode session"
                    )?;

                    session.current_refresh_jti = refresh_jti;

                    let encoded = trace_err!(
                        serialize_in_transaction_with_span(&self.bincode_config, &session),
                        "failed to bin encode session"
                    )?;

                    trace_err!(
                        insert_value_in_transaction_with_span(&key, &encoded, tx),
                        "failed to write session into storage"
                    )?;

                    Ok(())
                } else {
                    tracing::error!(session_id = %id, "failed to find session with id");
                    Err(SledStorageError::NoContent.into())
                }
            })
        })
        .map_err(SledStorageError::from)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
