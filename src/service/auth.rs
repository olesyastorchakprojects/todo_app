use std::sync::Arc;

use tracing::{info, info_span, instrument};

use crate::{
    config::JwtConfig,
    handlers::{error::AppError, LoginToken},
    service::jwt,
    storage::{Jti, Session, SessionId, SessionStorage},
    trace_err,
    utils::{measure_metrics::measure_and_record_service, JWT_SECRET_KEY},
};

pub struct ServiceAuthRef {
    storage: Arc<dyn SessionStorage>,
}

impl ServiceAuthRef {
    pub(crate) fn new(storage: Arc<dyn SessionStorage>) -> Self {
        Self { storage }
    }

    #[instrument(name = "Service::Session::get", skip_all)]
    pub(crate) async fn get(&self, id: SessionId) -> Result<Session, AppError> {
        info!(session_id = %id, "get session");

        measure_and_record_service("get_session", || async { self.storage.get(id).await })
            .await
            .map_err(Into::into)
    }

    #[instrument(name = "Service::session::add", skip_all)]
    pub(crate) async fn add(&self, session: Session) -> Result<(), AppError> {
        info!(session_id = %session.id, "create session");

        measure_and_record_service("add_session", || async {
            self.storage.put(session.id, session).await
        })
        .await?;

        Ok(())
    }

    #[instrument(name = "Service::session::delete", skip_all)]
    pub(crate) async fn delete(&self, id: SessionId) -> Result<(), AppError> {
        info!(session_id = %id, "delete session");

        measure_and_record_service("add_session", || async { self.storage.delete(id).await })
            .await?;

        Ok(())
    }

    #[instrument(name = "Service::session::refresh_token", skip_all)]
    pub(crate) async fn refresh_token(
        &self,
        session: &Session,
        jwt_config: &JwtConfig,
    ) -> Result<LoginToken, AppError> {
        measure_and_record_service("refresh_token", || async {
            let refresh_jti = Jti::new();
            let jwt_secret = std::env::var(JWT_SECRET_KEY).map_err(|e|{
                tracing::error!(error = %e, "failed to load JWT_SECRET_KEY env var");
                AppError::FailedToLoadEnvVar(JWT_SECRET_KEY)})?;

            let access_token = info_span!("generate_access_token").in_scope(|| {
                info!(user_id = %session.user_id, ttl_sec = jwt_config.access_token_ttl_sec, "generating access token");
                trace_err!(jwt::generate_access_token(
                    session.user_id,
                    session.id,
                    &jwt_secret,
                    jwt_config.access_token_ttl_sec,
                ), "failed to generate jwt access token")
            })?;

            let refresh_token = info_span!("generate_refresh_token").in_scope(|| {
                info!(user_id = %session.user_id, ttl_days = jwt_config.refresh_token_ttl_sec, "generating refresh token");
                trace_err!(jwt::generate_refresh_token(
                    session.user_id,
                    session.id,
                    refresh_jti,
                    &jwt_secret,
                    jwt_config.refresh_token_ttl_sec,
                ), "failed to generate refresh token")
            })?;

            self.storage.update(session.id, refresh_jti).await?;

            Ok(LoginToken{access_token, refresh_token})
        })
        .await
    }
}
