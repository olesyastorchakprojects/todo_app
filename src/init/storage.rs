use crate::{
    config::types::StorageKind,
    service::Service,
    storage::{FlushStorage, SessionStorage, TodoStorage, UserStorage},
    Settings,
};
use std::sync::Arc;

use tracing::instrument;

use crate::storage::SledStorage;

use super::StartupError;

#[instrument(name = "init_storage")]
pub async fn init_storage(settings: &Settings) -> Result<Service, StartupError> {
    let service = match &settings.storage.backend {
        StorageKind::Sled => {
            let sled_storage = Arc::new(
                SledStorage::new(
                    settings
                        .storage
                        .sled
                        .as_ref()
                        .ok_or(StartupError::MissingStorageConfig("sled".to_string()))?,
                )
                .map_err(StartupError::OpenSledStorage)?,
            );

            Service::new(
                sled_storage.clone() as Arc<dyn TodoStorage>,
                sled_storage.clone() as Arc<dyn UserStorage>,
                sled_storage.clone() as Arc<dyn SessionStorage>,
                sled_storage.clone() as Arc<dyn FlushStorage>,
            )
            .await
        }
        kind => {
            return Err(StartupError::UnsupportedStorage(kind.as_ref().to_string()));
        }
    };

    service.user().create_admins(settings).await?;

    Ok(service)
}
