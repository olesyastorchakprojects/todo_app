pub(super) mod error;
mod flush_impl;
mod internal;
mod session_impl;
mod todos_impl;
mod users_impl;

#[cfg(feature = "integration_tests")]
pub mod test_util;

use super::{
    Pagination, Session, SessionId, StorageError, Todo, TodoId, TodoStorage, TodoVersion,
    UpdateTodo, User, UserId, UserStorage,
};
use crate::{config::types::SledConfig, utils::measure_metrics::measure_and_record_storage};
use bincode::config::{self};
use error::{SledStartupError, SledStorageError};
use internal::{Key, KeyPrefix, PrefixKind};
use tracing::{info, info_span, instrument};

pub(crate) static SLED_TODO_TREE: &str = "todos";
pub(crate) static SLED_USER_TREE: &str = "users";
pub(crate) static SLED_EMAIL_TREE: &str = "emails";
pub(crate) static SLED_SESSION_TREE: &str = "sessions";
const BINCODE_CONFIG: config::Configuration = config::standard()
    .with_variable_int_encoding()
    .with_little_endian();

use bincode::{Decode, Encode};

type BincodeConfig = bincode::config::Configuration;

trait ToBytesWithConfig: Encode {
    type Error;

    fn to_bytes(&self, config: &BincodeConfig) -> Result<Vec<u8>, Self::Error>;
}

trait FromBytesWithConfig: Decode<()> {
    type Error;

    fn from_bytes(bytes: &[u8], config: &BincodeConfig) -> Result<Self, Self::Error>;
}

pub(crate) struct SledStorage {
    todo_tree: sled::Tree,
    user_tree: sled::Tree,
    email_tree: sled::Tree,
    session_tree: sled::Tree,
    bincode_config: config::Configuration,
    storage_settings: SledConfig,
}

impl SledStorage {
    #[instrument(name = "Storage::new")]
    pub fn new(sled_config: &SledConfig) -> Result<Self, SledStartupError> {
        let result: Result<_, SledStartupError> = measure_and_record_storage(
            "Storage::new",
            || {
                let db = info_span!("sled::open_db").in_scope(|| {
                    let config = sled::Config::default().path(&sled_config.path);
                    config.open().map_err(|e| {
                        tracing::error!(error = %e, path = ?sled_config.path,"failed to open db");
                        SledStartupError::OpenSledStorageError(e)
                    })
                })?;

                let todo_tree = info_span!("sled::open_todo_tree").in_scope(|| {
                    db.open_tree(SLED_TODO_TREE).map_err(|e| {
                        tracing::error!(error = %e, tree_name = SLED_TODO_TREE, "failed to open todo tree");
                        SledStartupError::OpenSledStorageError(e)
                    })
                })?;

                let user_tree = info_span!("sled::open_user_tree").in_scope(|| {
                    db.open_tree(SLED_USER_TREE)
                        .map_err(|e| {
                            tracing::error!(error = %e, tree_name = SLED_USER_TREE, "failed to open user tree");
                            SledStartupError::OpenSledStorageError(e) })
                })?;

                let email_tree = info_span!("sled::open_email_tree").in_scope(|| {
                    db.open_tree(SLED_EMAIL_TREE)
                        .map_err(|e| {
                            tracing::error!(error = %e, tree_name = SLED_EMAIL_TREE, "failed to open email tree");
                            SledStartupError::OpenSledStorageError(e)})
                })?;

                let session_tree = info_span!("sled::open_session_tree").in_scope(|| {
                    db.open_tree(SLED_SESSION_TREE)
                        .map_err(|e| {
                            tracing::error!(error = %e, tree_name = SLED_SESSION_TREE, "failed to open session tree");
                            SledStartupError::OpenSledStorageError(e)})
                })?;

                Ok(Self {
                    todo_tree,
                    user_tree,
                    email_tree,
                    session_tree,
                    bincode_config: BINCODE_CONFIG,
                    storage_settings: sled_config.clone(),
                })
            },
        );
        result
    }
}

fn todo_key(user_id: &UserId, todo_id: &TodoId) -> Key {
    Key::new(KeyPrefix::new(PrefixKind::Todo, user_id), todo_id)
}

fn user_key(user_id: &UserId) -> Key {
    Key::new(KeyPrefix::from_kind(PrefixKind::User), user_id)
}

fn email_key(email: &str) -> Key {
    Key::new(KeyPrefix::from_kind(PrefixKind::Email), email)
}

fn session_key(session_id: &SessionId) -> Key {
    Key::new(KeyPrefix::from_kind(PrefixKind::Session), session_id)
}

impl ToBytesWithConfig for User {
    type Error = SledStorageError;

    #[instrument(name = "User::to_bytes", skip_all)]
    fn to_bytes(&self, config: &BincodeConfig) -> Result<Vec<u8>, Self::Error> {
        Ok(bincode::encode_to_vec(self, *config)?)
    }
}

impl FromBytesWithConfig for User {
    type Error = SledStorageError;

    #[instrument(name = "User::from_bytes", skip_all)]
    fn from_bytes(bytes: &[u8], config: &BincodeConfig) -> Result<Self, Self::Error> {
        let (user, _len) = bincode::decode_from_slice::<User, _>(bytes, *config)?;

        info!(user_email = %user.email, "created User from bytes");

        Ok(user)
    }
}

impl FromBytesWithConfig for TodoVersion {
    type Error = SledStorageError;

    #[instrument(name = "TodoVersion::from_bytes", skip_all)]
    fn from_bytes(bytes: &[u8], config: &BincodeConfig) -> Result<Self, Self::Error> {
        let (todo, _len) = bincode::decode_from_slice::<TodoVersion, _>(bytes, *config)?;
        Ok(todo)
    }
}

impl ToBytesWithConfig for TodoVersion {
    type Error = SledStorageError;

    #[instrument(name = "TodoVersion::to_bytes", skip_all)]
    fn to_bytes(&self, config: &BincodeConfig) -> Result<Vec<u8>, Self::Error> {
        let bytes = bincode::encode_to_vec(self, *config)?;
        Ok(bytes)
    }
}

impl ToBytesWithConfig for Session {
    type Error = SledStorageError;

    #[instrument(name = "Session::to_bytes", skip_all)]
    fn to_bytes(&self, config: &BincodeConfig) -> Result<Vec<u8>, Self::Error> {
        Ok(bincode::encode_to_vec(self, *config)?)
    }
}

impl FromBytesWithConfig for Session {
    type Error = SledStorageError;

    #[instrument(name = "Session::from_bytes", skip_all)]
    fn from_bytes(bytes: &[u8], config: &BincodeConfig) -> Result<Self, Self::Error> {
        let (session, _len) = bincode::decode_from_slice::<Session, _>(bytes, *config)?;

        info!(session_id = %session.id, "created Session obj from bytes");

        Ok(session)
    }
}
