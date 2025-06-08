use crate::storage::user::Role;
use crate::storage::{Pagination, UserId};
use crate::utils::blocking_task_guard::BlockingTaskGuard;
use crate::utils::measure_metrics::measure_and_record_storage;

use super::error::SledStorageError;
use super::internal::span_wrappers::remove_value_in_transaction_with_span;
use super::internal::TreeScan;
use super::internal::{
    span_wrappers::{
        deserialize_in_span, deserialize_in_transaction_with_span,
        get_value_in_transaction_with_span, get_value_with_span,
        insert_value_in_transaction_with_span, remove_batch_in_transaction_with_span,
        serialize_in_transaction_with_span,
    },
    Key, KeyPrefix, PrefixKind,
};
use super::{email_key, BincodeConfig, SledStorage};
use super::{user_key, FromBytesWithConfig};
use super::{StorageError, User, UserStorage};
use crate::trace_err;
use async_trait::async_trait;
use sled::transaction::ConflictableTransactionError;
use sled::{Transactional, Tree};
use tracing::{info, info_span, instrument, Span};

#[async_trait]
impl UserStorage for SledStorage {
    #[instrument(name = "SledStorage::create_user", skip_all)]
    async fn put(&self, user_id: UserId, user: User) -> Result<(), StorageError> {
        // cloning tree should be cheap: struct Tree{inner: Arc<TreeInner>}
        let (user_tree, email_tree, bincode_config) = info_span!("Cloning trees and config")
            .in_scope(|| {
                (
                    self.user_tree.clone(),
                    self.email_tree.clone(),
                    self.bincode_config.clone(),
                )
            });

        let span = Span::current();
        tokio::task::spawn_blocking(move || {
            let _guard = BlockingTaskGuard::new("add_new_user");
            span.in_scope(|| add_user(user_id, user, &user_tree, &email_tree, &bincode_config))
        })
        .await?
    }

    #[instrument(name = "SledStorage::get_user_by_email", skip_all)]
    async fn get_by_email(&self, email: &str) -> Result<User, StorageError> {
        info!(email = %email, "get user by email");

        let result: Result<User, SledStorageError> =
            measure_and_record_storage("SledStorage::get_user_by_email", || {
                let key = email_key(email);
                let value = trace_err!(
                    get_value_with_span(&key, &self.email_tree),
                    "failed to read user from emails tree"
                )?;

                trace_err!(
                    deserialize_in_span(&self.bincode_config, &value),
                    "failed to bin decode user"
                )
            });

        Ok(result?)
    }

    #[instrument(name = "SledStorage::get_user_by_id", skip_all)]
    async fn get(&self, user_id: UserId) -> Result<User, StorageError> {
        info!(user_id = %user_id, "get user");

        let result: Result<User, SledStorageError> =
            measure_and_record_storage("SledStorage::get_user_by_id", || {
                let key = user_key(&user_id);
                let value = trace_err!(
                    get_value_with_span(&key, &self.user_tree),
                    "failed to read user from users tree"
                )?;

                trace_err!(
                    deserialize_in_span(&self.bincode_config, &value),
                    "failed to bin decode user"
                )
            });

        Ok(result?)
    }

    #[instrument(name = "SledStorage::delete_user", skip_all)]
    async fn delete(&self, user_id: UserId) -> Result<(), StorageError> {
        info!(user_id = %user_id, "delete user");

        let result: Result<_, SledStorageError> =
            measure_and_record_storage("SledStorage::delete_user", || {
                {
                    let _ = info_span!("remove user and todos in transaction", user_id = ?user_id)
                        .entered();
                    let user_key = user_key(&user_id);
                    let todos_key_prefix = KeyPrefix::new(PrefixKind::Todo, user_id);
                    let first_key = Key::new(KeyPrefix::from_kind(PrefixKind::Todo), user_id);

                    if let Err(SledStorageError::NotFound) =
                        get_value_with_span(&user_key, &self.user_tree)
                    {
                        tracing::error!(user_id = %user_id, "failed to find user in users tree");
                        return Err(SledStorageError::NoContent);
                    }

                    let mut after: Option<Key> = None;
                    loop {
                        let after_key = after.as_ref().map_or(&first_key, |v| v);

                        let mut page = trace_err!(
                            TreeScan::scan_from(&self.todo_tree, after_key)
                                .within(todos_key_prefix.clone())
                                .with_pagination(Pagination {
                                    after: after.clone(),
                                    limit: self.storage_settings.delete_batch_size,
                                })
                                .collect(&self.bincode_config, |key, _, _| Ok(key.clone()), None),
                            "failed to do tree scan to get page of users todo to delete"
                        )?;

                        // Delete cursor from prev iteration
                        if let Some(prev_cursor) = after {
                            page.items.insert(0, prev_cursor);
                        }
                        // Keep cursor for next iteration
                        if page.next_cursor.is_some() {
                            page.items.pop();
                        }
                        (&self.user_tree, &self.email_tree, &self.todo_tree).transaction(
                            |(user_tree, email_tree, todo_tree)| {
                                trace_err!(
                                    remove_batch_in_transaction_with_span(&page.items, todo_tree),
                                    "failed to remove page of user todo-s"
                                )?;

                                if page.next_cursor.is_none() {
                                    trace_err!(
                                        self.remove_user_and_email(
                                            user_tree, email_tree, &user_key
                                        ),
                                        "failed to remove user records in users and emails trees"
                                    )?;
                                }
                                Ok(())
                            },
                        )?;

                        match page.next_cursor {
                            Some(cursor) => after = Some(cursor),
                            None => break,
                        }
                    }
                }
                Ok(())
            });

        Ok(result?)
    }

    #[instrument(name = "SledStorage::get_users", skip_all)]
    async fn get_all(
        &self,
        user_id: UserId,
        pagination: Pagination<UserId>,
    ) -> Result<(Vec<User>, Option<UserId>), StorageError> {
        info!(pagination = ?pagination, "get all users");

        let result: Result<_, SledStorageError> =
            measure_and_record_storage("SledStorage::get_users", || {
                let after_key = match pagination.after {
                    Some(user_id) => user_key(&user_id),
                    None => Key::from_prefix(KeyPrefix::from_kind(PrefixKind::User)),
                };

                let page = info_span!("TreeScan::scan_from::within::until_pagination::collect")
                    .in_scope(|| {
                        let user_filter = |user: &User| user.id != user_id;
                        trace_err!(
                            TreeScan::scan_from(&self.user_tree, &after_key)
                                .within(KeyPrefix::from_kind(PrefixKind::User))
                                .with_pagination(pagination)
                                .collect(
                                    &self.bincode_config,
                                    |_, bytes, config| User::from_bytes(bytes, config),
                                    Some(&user_filter),
                                ),
                            "failed to do tree scan to get page of users"
                        )
                    })?;
                Ok((page.items, page.next_cursor))
            });

        Ok(result?)
    }

    #[instrument(name = "SledStorage::change_user_role", skip_all)]
    async fn update_role(&self, user_id: UserId, role: Role) -> Result<(), StorageError> {
        // cloning tree should be cheap: struct Tree{inner: Arc<TreeInner>}
        let (user_tree, email_tree, bincode_config) = info_span!("Cloning trees and config")
            .in_scope(|| {
                (
                    self.user_tree.clone(),
                    self.email_tree.clone(),
                    self.bincode_config.clone(),
                )
            });

        let span = Span::current();
        tokio::task::spawn_blocking(move || {
            let _guard = BlockingTaskGuard::new("update_user_role");
            span.in_scope(|| {
                update_user_role(user_id, role, &user_tree, &email_tree, &bincode_config)
            })
        })
        .await?
    }
}

impl SledStorage {
    #[instrument(name = "SledStorage::remove_user_and_email", skip_all)]
    fn remove_user_and_email(
        &self,
        user_tree: &sled::transaction::TransactionalTree,
        email_tree: &sled::transaction::TransactionalTree,
        user_key: &Key,
    ) -> Result<(), SledStorageError> {
        info!(key = %user_key, "remove user record with user_id and email keys");

        let value = get_value_in_transaction_with_span(user_key, user_tree)?;
        if let Some(value) = value {
            let user: User = deserialize_in_transaction_with_span(&self.bincode_config, &value)?;

            let email_key = email_key(&user.email);

            remove_value_in_transaction_with_span(user_key, user_tree)?;
            remove_value_in_transaction_with_span(&email_key, email_tree)?;
        }

        Ok(())
    }
}

#[instrument(name = "SledStorage::add_user", skip_all)]
fn add_user(
    user_id: UserId,
    user: User,
    user_tree: &Tree,
    email_tree: &Tree,
    bincode_config: &BincodeConfig,
) -> Result<(), StorageError> {
    info!(user_id = %user_id, user = ?user, "create user");

    measure_and_record_storage("SledStorage::add_new_user", || {
        let key_user_id = user_key(&user_id);
        let key_email = email_key(&user.email);

        info_span!("sled::add_new_user_in_transaction", user = ?user).in_scope(|| {
            let res = (user_tree, email_tree).transaction(|(users_tx, emails_tx)| {
                let encoded: Vec<u8> = trace_err!(
                    serialize_in_transaction_with_span(bincode_config, &user),
                    "failed to bin encode user"
                )?;

                trace_err!(
                    insert_value_in_transaction_with_span(&key_user_id, &encoded, users_tx),
                    "failed to insert user record into users tree"
                )?;
                trace_err!(
                    insert_value_in_transaction_with_span(&key_email, &encoded, emails_tx),
                    "failed to insert user record into emails tree"
                )?;

                Ok(())
            });

            res
        })
    })
    .map_err(SledStorageError::from)?;
    Ok(())
}

#[instrument(name = "SledStorage::update_user_role", skip_all)]
fn update_user_role(
    user_id: UserId,
    role: Role,
    user_tree: &Tree,
    email_tree: &Tree,
    bincode_config: &BincodeConfig,
) -> Result<(), StorageError> {
    info!(user_id = %user_id, role = ?role, "update user role");

    measure_and_record_storage("SledStorage::change_user_role", || {
        let key = user_key(&user_id);
        info_span!("change role in transaction").in_scope(|| {
            (user_tree, email_tree).transaction(|(user_tx, email_tx)| {
                let value = trace_err!(
                    get_value_in_transaction_with_span(&key, user_tx),
                    "failed to read user from users tree"
                )?;

                if let Some(value) = value {
                    let mut user: User = trace_err!(
                        deserialize_in_transaction_with_span(bincode_config, &value),
                        "failed to bin decode user"
                    )?;

                    user.role = role;

                    let encode = trace_err!(
                        serialize_in_transaction_with_span(bincode_config, &user),
                        "failed to bin encode user"
                    )?;
                    let email_key = email_key(&user.email);

                    trace_err!(
                        insert_value_in_transaction_with_span(&key, &encode, user_tx),
                        "failed to insert user in users tree"
                    )?;
                    trace_err!(
                        insert_value_in_transaction_with_span(&email_key, &encode, email_tx),
                        "failed to insert user in emails tree"
                    )?;

                    Ok(())
                } else {
                    tracing::error!(user_id = %user_id, "failed to find user in users tree");

                    Err(ConflictableTransactionError::Abort(
                        SledStorageError::NotFound,
                    ))
                }
            })
        })
    })
    .map_err(SledStorageError::from)?;
    Ok(())
}

#[cfg(test)]
mod tests;
