use ring::pbkdf2::{self, PBKDF2_HMAC_SHA256};
use std::num::NonZeroU32;
use tracing::instrument;

use rand::{rng, RngCore};

use crate::{
    config::types::Pbkdf2Config,
    handlers::error::AppError,
    storage::{HashedPassword, HASH_LEN, SALT_LEN},
    utils::{
        blocking_task_guard::BlockingTaskGuard,
        measure_metrics::{measure_and_record_password_hash, measure_and_record_password_verify},
    },
};

#[instrument(name = "create_hashed_password_pbkdf2", skip_all)]
pub async fn create_hashed_password_pkdf2b(
    password: &str,
    pkdf2b_config: &Pbkdf2Config,
) -> Result<HashedPassword, AppError> {
    let password = password.to_string();
    let pkdf2b_config = pkdf2b_config.clone();

    let mut salt = [0u8; SALT_LEN];
    rng().fill_bytes(&mut salt);

    let hash = tokio::task::spawn_blocking(move || {
        let _guard = BlockingTaskGuard::new("create_password_pbkdf2");

        measure_and_record_password_hash("pbkdf2", || {
            tracing::info_span!("pbkdf2").in_scope(|| {
                let mut hash = [0u8; HASH_LEN];
                pbkdf2::derive(
                    PBKDF2_HMAC_SHA256,
                    NonZeroU32::new(pkdf2b_config.iterations).expect("ITERATIONS must be > 0"),
                    &salt,
                    password.as_bytes(),
                    &mut hash,
                );
                Ok::<Vec<u8>, AppError>(hash.to_vec())
            })
        })
    })
    .await
    .map_err(AppError::from)??;

    Ok(HashedPassword {
        salt: salt.to_vec(),
        hash,
    })
}

#[instrument(name = "verify_password_pbkdf2", skip_all)]
pub async fn verify_password_pbkdf2(
    password: &str,
    stored: &HashedPassword,
    pkdf2b_config: &Pbkdf2Config,
) -> Result<(), AppError> {
    let password = password.to_string();
    let pkdf2b_config = pkdf2b_config.clone();
    let stored = stored.clone();

    tokio::task::spawn_blocking(move || {
        let _guard = BlockingTaskGuard::new("verify_password_pbkdf2");

        measure_and_record_password_verify("pbkdf2", || {
            tracing::info_span!("pbkdf2").in_scope(|| {
                pbkdf2::verify(
                    PBKDF2_HMAC_SHA256,
                    NonZeroU32::new(pkdf2b_config.iterations).expect("ITERATIONS must be > 0"),
                    &stored.salt,
                    password.as_bytes(),
                    &stored.hash,
                )
                .map_err(|_| AppError::PasswordMismatch)?;
                Ok::<(), AppError>(())
            })
        })
    })
    .await??;

    Ok(())
}
