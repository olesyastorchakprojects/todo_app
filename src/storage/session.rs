use bincode::{Decode, Encode};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::config::JwtConfig;

use super::{Jti, SessionId, StorageError, UserId};

#[derive(Encode, Decode, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub created_at: i64,
    pub expires_at: i64,
    pub current_refresh_jti: Jti,
}

impl Session {
    pub(crate) fn new(
        user_id: &UserId,
        refresh_jti: &Jti,
        jwt_config: &JwtConfig,
    ) -> Result<Self, StorageError> {
        let created_at = Utc::now();
        Ok(Self {
            id: SessionId::new(),
            user_id: *user_id,
            created_at: created_at.timestamp(),
            expires_at: created_at
                .checked_add_signed(chrono::Duration::seconds(jwt_config.session_ttl_sec))
                .ok_or(StorageError::InvalidTtl)?
                .timestamp(),
            current_refresh_jti: *refresh_jti,
        })
    }

    pub(crate) fn validate(&self) -> Result<(), StorageError> {
        (self.expires_at > Utc::now().timestamp())
            .then_some(())
            .ok_or(StorageError::SessionExpired)
    }
}
