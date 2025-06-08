use tracing::{info, instrument, warn};

use crate::storage::sled::{
    error::SledStorageError, internal::Key, BincodeConfig, FromBytesWithConfig, ToBytesWithConfig,
};

#[instrument(name = "sled::get_value_by_key", skip_all)]
pub(crate) fn get_value_with_span(
    key: &Key,
    tree: &sled::Tree,
) -> Result<sled::IVec, SledStorageError> {
    info!(key = ?key, "get value with key");
    tree.get(key.as_bytes())?.ok_or(SledStorageError::NotFound)
}

#[instrument(name = "sled::insert_value_with_key", skip_all)]
pub(crate) fn insert_value_with_span(
    key: &Key,
    value: &[u8],
    tree: &sled::Tree,
) -> Result<(), SledStorageError> {
    info!(key = ?key, "insert value with key");
    let old_value = tree.insert(key.as_bytes(), value)?;
    if old_value.is_some() {
        warn!(key = %key, "insert replaced old value");
    }

    Ok(())
}

#[instrument(name = "sled::remove_value_with_key", skip_all)]
pub(crate) fn remove_value_with_span(key: &Key, tree: &sled::Tree) -> Result<(), SledStorageError> {
    info!(key = ?key, "remove value with key");
    if tree.remove(key.as_bytes())?.is_none() {
        warn!(key = ?key, "Tried to remove non-existing key");
        Err(SledStorageError::NoContent)
    } else {
        Ok(())
    }
}

#[instrument(name = "sled::get_value_by_key", skip_all)]
pub(crate) fn get_value_in_transaction_with_span(
    key: &Key,
    tree: &sled::transaction::TransactionalTree,
) -> Result<Option<sled::IVec>, SledStorageError> {
    info!(key = ?key, "get value with key");
    Ok(tree.get(key.as_bytes())?)
}

#[instrument(name = "sled::insert_value_with_key", skip_all)]
pub(crate) fn insert_value_in_transaction_with_span(
    key: &Key,
    value: &[u8],
    tree: &sled::transaction::TransactionalTree,
) -> Result<(), SledStorageError> {
    info!(key = ?key, "insert value with key");

    let old_value = tree.insert(key.as_bytes(), value)?;

    if old_value.is_some() {
        warn!(key = %key, "insert replaced old value");
    }
    Ok(())
}

#[instrument(name = "sled::remove_value_with_key", skip_all)]
pub(crate) fn remove_value_in_transaction_with_span(
    key: &Key,
    tree: &sled::transaction::TransactionalTree,
) -> Result<(), SledStorageError> {
    info!(key = ?key, "remove value with key");
    if tree.remove(key.as_bytes())?.is_none() {
        warn!(key = ?key, "Tried to remove non-existing key");
    }
    Ok(())
}

#[instrument(name = "sled::remove_batch", skip_all)]
pub(crate) fn remove_batch_in_transaction_with_span(
    keys: &[Key],
    tree: &sled::transaction::TransactionalTree,
) -> Result<(), SledStorageError> {
    let mut batch = sled::Batch::default();
    for key in keys.iter() {
        batch.remove(key.as_bytes());
    }

    tree.apply_batch(&batch)?;
    info!(removed_items = keys.len(), "removed items");
    Ok(())
}

#[instrument(name = "convert_value_to_bytes", skip_all)]
pub(crate) fn serialize_in_transaction_with_span(
    config: &BincodeConfig,
    value: &impl ToBytesWithConfig<Error = SledStorageError>,
) -> Result<Vec<u8>, SledStorageError> {
    value.to_bytes(config)
}

#[instrument(name = "convert_bytes_to_value", skip_all)]
pub(crate) fn deserialize_in_transaction_with_span<
    T: FromBytesWithConfig<Error = SledStorageError>,
>(
    config: &BincodeConfig,
    bytes: &[u8],
) -> Result<T, SledStorageError> {
    T::from_bytes(bytes, config)
}

#[instrument(name = "convert_bytes_to_value", skip_all)]
pub(crate) fn deserialize_in_span<T: FromBytesWithConfig<Error = SledStorageError>>(
    config: &BincodeConfig,
    bytes: &[u8],
) -> Result<T, SledStorageError> {
    T::from_bytes(bytes, config)
}

#[instrument(name = "convert_value_to_bytes", skip_all)]
pub(crate) fn serialize_in_span<T: ToBytesWithConfig<Error = SledStorageError>>(
    config: &BincodeConfig,
    value: &T,
) -> Result<Vec<u8>, SledStorageError> {
    value.to_bytes(config)
}

#[instrument(name = "flush_tree", skip_all)]
pub(crate) fn flush_tree_in_span(
    tree: &sled::Tree,
    tree_name: &'static str,
) -> Result<(), SledStorageError> {
    let bytes = tree.flush()?;
    info!(bytes = %bytes, tree_name = %tree_name, "flushed sled tree");

    Ok(())
}
