use std::fmt::Display;

use crate::storage::{
    page::{HasId, Page},
    sled::error::SledStorageError,
    Pagination,
};

use super::{Key, KeyPrefix};
use bincode::config;
use sled::Tree;
use tracing::{error, info, instrument};

pub(crate) struct TreeScan<'a, Id> {
    tree: &'a sled::Tree,
    after_key: &'a Key,
    prefix: Option<KeyPrefix>,
    pagination: Option<Pagination<Id>>,
}

impl<'a, Id> TreeScan<'a, Id> {
    pub fn scan_from(tree: &'a Tree, after_key: &'a Key) -> Self {
        Self {
            tree,
            after_key,
            prefix: None,
            pagination: None,
        }
    }

    pub fn within(mut self, prefix: KeyPrefix) -> Self {
        self.prefix = Some(prefix);
        self
    }

    pub fn with_pagination(mut self, pagination: Pagination<Id>) -> Self {
        self.pagination = Some(pagination);
        self
    }

    fn validate(&self) -> Result<(&Pagination<Id>, &KeyPrefix), SledStorageError> {
        let error = if self.pagination.is_none() && self.prefix.is_none() {
            "'within' and 'intil_pagination'"
        } else if self.pagination.is_none() {
            "'intil_pagination'"
        } else if self.prefix.is_none() {
            "'within'"
        } else {
            ""
        };
        Ok((
            self.pagination
                .as_ref()
                .ok_or(SledStorageError::UninitializedTreeScan(error))?,
            self.prefix
                .as_ref()
                .ok_or(SledStorageError::UninitializedTreeScan(error))?,
        ))
    }

    #[instrument(name = "TreeScan::collect", skip_all)]
    pub fn collect<T>(
        self,
        config: &config::Configuration,
        deserialize: impl Fn(&Key, &[u8], &config::Configuration) -> Result<T, SledStorageError>,
        filter: Option<&dyn Fn(&T) -> bool>,
    ) -> Result<Page<T, Id>, SledStorageError>
    where
        T: HasId<Id>,
        Id: Display,
    {
        let (pagination, prefix) = self.validate()?;

        info!(prefix = %prefix, "collect values with key prefix");
        if let Some(cursor) = &pagination.after {
            info!(after = %cursor, "collect values after key");

            self.after_key.exists_in(self.tree).inspect_err(|_| {
                error!(key = %self.after_key, "Key not found");
            })?;
        }

        let mut page = Page::from(pagination);
        let iter = self.tree.range(self.after_key.as_bytes()..);

        for item in iter {
            let (key_bytes, value_bytes) = item?;
            let key = Key::from_bytes(&key_bytes)?;

            if key == *self.after_key {
                continue;
            }

            if !key.starts_with(prefix) {
                break;
            }

            let deserialized_value = deserialize(&key, &value_bytes, config)?;
            if let Some(filter) = filter {
                if !filter(&deserialized_value) {
                    continue;
                }
            }

            if page.complete_with(deserialized_value) {
                break;
            }
        }

        Ok(page)
    }
}
