use std::fmt::Display;
use std::str::FromStr;

use sled::Tree;
use tracing::{debug, instrument};

use crate::storage::{page::HasId, sled::error::SledStorageError};
use strum::AsRefStr;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Debug, EnumString, EnumIter, AsRefStr, Display, PartialEq, Eq, Copy, Clone)]
#[strum(serialize_all = "lowercase")]
pub(crate) enum PrefixKind {
    User,
    Email,
    Todo,
    Session,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KeyPrefix {
    prefix: String,
}

impl KeyPrefix {
    pub fn new(kind: PrefixKind, value: impl Display) -> Self {
        Self {
            prefix: format!("{}:{}:", kind.as_ref(), value),
        }
    }

    pub fn from_kind(kind: PrefixKind) -> Self {
        Self {
            prefix: format!("{}:", kind.as_ref()),
        }
    }

    pub fn from_parts(parts: &[&str]) -> Self {
        Self {
            prefix: parts.join(":") + ":",
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        self.prefix.as_str()
    }
}

impl std::fmt::Display for KeyPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.prefix.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Key {
    prefix: KeyPrefix,
    full_key: String,
}

impl Key {
    pub fn new(prefix: KeyPrefix, value: impl Display) -> Self {
        Self {
            full_key: format!("{}{}", prefix.as_str(), value),
            prefix,
        }
    }

    pub fn from_prefix(prefix: KeyPrefix) -> Self {
        Self {
            full_key: prefix.as_str().to_string(),
            prefix,
        }
    }

    #[instrument(name = "Key::from_bytes", skip_all, level = "debug")]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SledStorageError> {
        let full_key = std::str::from_utf8(bytes)
            .map_err(SledStorageError::Conversion)?
            .to_string();

        let parts: Vec<&str> = full_key.split(':').collect();

        if parts.len() >= 2 {
            if PrefixKind::from_str(parts[0]).is_err() {
                return Err(SledStorageError::InvalidKey(full_key));
            }
            if parts.iter().any(|v| v.is_empty()) {
                return Err(SledStorageError::InvalidKey(full_key));
            }
            let prefix = KeyPrefix::from_parts(&parts[..parts.len() - 1]);

            debug!(key = %full_key, prefix = %prefix, "created key from bytes");

            Ok(Self { prefix, full_key })
        } else {
            Err(SledStorageError::InvalidKey(full_key))
        }
    }

    pub fn exists_in(&self, tree: &Tree) -> Result<(), SledStorageError> {
        if tree.get(self.as_bytes())?.is_none() {
            return Err(SledStorageError::NotFound);
        };
        Ok(())
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.full_key.as_bytes()
    }

    pub fn starts_with(&self, prefix: &KeyPrefix) -> bool {
        self.full_key.starts_with(prefix.as_str())
    }
}

impl HasId<Key> for Key {
    fn id(&self) -> Key {
        self.clone()
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.full_key.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key() {
        let key1 = Key::new(KeyPrefix::new(PrefixKind::User, "xxx"), "xxx");
        assert_eq!(key1.full_key, "user:xxx:xxx");

        let key2 = Key::new(KeyPrefix::from_kind(PrefixKind::User), "xxx");
        assert_eq!(key2.full_key, "user:xxx");

        let key3 = Key::new(KeyPrefix::from_parts(&["todo", "xxx"]), "xxx");
        assert_eq!(key3.full_key, "todo:xxx:xxx");

        let key4 = Key::from_prefix(KeyPrefix::from_parts(&["todo", "xxx"]));
        assert_eq!(key4.full_key, "todo:xxx:");

        let key5 = Key::from_prefix(KeyPrefix::from_kind(PrefixKind::Email));
        assert_eq!(key5.full_key, "email:");

        let key6 = Key::from_bytes("todo:xxx:xxx".as_bytes()).unwrap();
        assert_eq!(key6.prefix.as_str(), "todo:xxx:");

        let key7 = Key::from_bytes("todo:xxx".as_bytes()).unwrap();
        assert_eq!(key7.prefix.as_str(), "todo:");

        let key8 = Key::from_bytes("todo".as_bytes());
        assert!(key8.is_err());

        let key9 = Key::from_bytes("ddd:fff:eee".as_bytes());
        assert!(key9.is_err());
    }
}
