use crate::trace_err;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, ParamsBuilder,
};
use tracing::instrument;

use crate::{
    config::types::Argon2Config,
    handlers::error::AppError,
    storage::HashedPassword,
    utils::{
        blocking_task_guard::BlockingTaskGuard,
        measure_metrics::{measure_and_record_password_hash, measure_and_record_password_verify},
    },
};

fn argon(argon2_config: &Argon2Config) -> Result<Argon2, AppError> {
    Ok(Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        trace_err!(
            ParamsBuilder::new()
                .m_cost(argon2_config.memory_cost)
                .t_cost(argon2_config.time_cost)
                .p_cost(argon2_config.parallelism)
                .build(),
            "failed to build Argon2 with Param builder"
        )?,
    ))
}

#[instrument(name = "create_hashed_password_argon2", skip_all)]
pub async fn create_hashed_password_argon2(
    password: &str,
    argon2_config: &Argon2Config,
) -> Result<HashedPassword, AppError> {
    let password = password.to_string();
    let argon2_config = argon2_config.clone();

    let password_hash = tokio::task::spawn_blocking(move || {
        let _guard = BlockingTaskGuard::new("create_password_argon2");

        measure_and_record_password_hash("argon2", || {
            let salt = SaltString::generate(&mut OsRng);

            Ok::<String, AppError>(
                trace_err!(
                    argon(&argon2_config)?.hash_password(password.as_bytes(), &salt),
                    "failed to hash password"
                )?
                .to_string(),
            )
        })
    })
    .await
    .map_err(AppError::from)??;

    Ok(HashedPassword {
        hash: password_hash.as_bytes().to_vec(),
        salt: vec![], // we don't need to save salt for verification for argon2 algo
    })
}

#[instrument(name = "verify_password_argon2", skip_all)]
pub async fn verify_password_argon2(
    password: &str,
    stored: &HashedPassword,
    argon2_config: &Argon2Config,
) -> Result<(), AppError> {
    let stored_hash = String::from_utf8(stored.hash.clone())?;
    let password = password.to_string();
    let argon2_config = argon2_config.clone();

    tokio::task::spawn_blocking(move || {
        let _guard = BlockingTaskGuard::new("verify_password_argon2");

        measure_and_record_password_verify("argon2", || {
            let parsed_hash = PasswordHash::new(&stored_hash)?;

            argon(&argon2_config)?
                .verify_password(password.as_bytes(), &parsed_hash)
                .map_err(|e| {
                    tracing::error!(error = %e, "failed to verify password");
                    AppError::PasswordMismatch
                })
        })
    })
    .await??;

    Ok(())
}
