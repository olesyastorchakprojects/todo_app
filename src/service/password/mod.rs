use crate::{
    config::types::{AuthSettings, KDFKind},
    handlers::error::AppError,
    storage::HashedPassword,
};

pub(crate) mod argon2;
pub(crate) mod pbkdf2;

pub async fn create_password_hash(
    password: &str,
    auth_config: &AuthSettings,
) -> Result<HashedPassword, AppError> {
    match auth_config.kdf_algo {
        KDFKind::Argon2 => {
            argon2::create_hashed_password_argon2(
                password,
                auth_config
                    .argon2
                    .as_ref()
                    .ok_or(AppError::MissingArgon2Config)?,
            )
            .await
        }
        KDFKind::Pbkdf2 => {
            pbkdf2::create_hashed_password_pkdf2b(
                password,
                auth_config
                    .pbkdf2
                    .as_ref()
                    .ok_or(AppError::MissingPbkdf2Config)?,
            )
            .await
        }
    }
}

pub async fn verify_password(
    password: &str,
    stored: &HashedPassword,
    auth_config: &AuthSettings,
) -> Result<(), AppError> {
    match auth_config.kdf_algo {
        KDFKind::Argon2 => {
            argon2::verify_password_argon2(
                password,
                stored,
                auth_config
                    .argon2
                    .as_ref()
                    .ok_or(AppError::MissingArgon2Config)?,
            )
            .await
        }
        KDFKind::Pbkdf2 => {
            pbkdf2::verify_password_pbkdf2(
                password,
                stored,
                auth_config
                    .pbkdf2
                    .as_ref()
                    .ok_or(AppError::MissingPbkdf2Config)?,
            )
            .await
        }
    }
}
