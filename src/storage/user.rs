use std::fmt::Debug;

use super::{page::HasId, UserId};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use strum::EnumString;

pub const SALT_LEN: usize = 20;
pub const HASH_LEN: usize = ring::digest::SHA256_OUTPUT_LEN;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Encode, Decode, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Role {
    User,
    Admin,
}

#[derive(Encode, Decode, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct HashedPassword {
    pub salt: Vec<u8>,
    pub hash: Vec<u8>,
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub hashed_password: HashedPassword,
    pub role: Role,
}

impl HasId<UserId> for User {
    fn id(&self) -> UserId {
        self.id
    }
}

impl std::fmt::Debug for HashedPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&"***", f)
    }
}

impl std::fmt::Display for HashedPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&"***", f)
    }
}
