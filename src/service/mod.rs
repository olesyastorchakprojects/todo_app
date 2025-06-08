pub(crate) mod auth;
pub(crate) mod jwt;
pub(crate) mod password;
pub(crate) mod todo;
pub(crate) mod user;

use moka::future::Cache;
use std::sync::Arc;

use crate::{
    handlers::{LoginToken, LoginUser},
    storage::{FlushStorage, Jti, Session, SessionStorage, TodoStorage, User, UserId, UserStorage},
    trace_err,
    utils::{measure_metrics::measure_and_record_service, JWT_SECRET_KEY},
    Settings,
};
use auth::ServiceAuthRef;
use password::verify_password;
use todo::ServiceTodoRef;
use tracing::{info, info_span, instrument};
use user::ServiceUserRef;

use crate::handlers::error::AppError;

#[derive(Clone)]
pub struct UserCache {
    by_id: Cache<UserId, User>,
    by_email: Cache<String, User>,
}

#[derive(Clone)]
pub struct Service {
    todo_storage: Arc<dyn TodoStorage>,
    user_storage: Arc<dyn UserStorage>,
    session_storage: Arc<dyn SessionStorage>,
    flush_storage: Arc<dyn FlushStorage>,
    user_cache: Arc<UserCache>,
}

impl Service {
    #[instrument(name = "Service::new", skip_all)]
    pub async fn new(
        todo_storage: Arc<dyn TodoStorage>,
        user_storage: Arc<dyn UserStorage>,
        session_storage: Arc<dyn SessionStorage>,
        flush_storage: Arc<dyn FlushStorage>,
    ) -> Self {
        Self {
            todo_storage,
            user_storage,
            session_storage,
            flush_storage,
            user_cache: Arc::new(UserCache {
                by_id: Cache::new(10_000),
                by_email: Cache::new(10_000),
            }),
        }
    }

    pub fn todo(&self) -> ServiceTodoRef {
        ServiceTodoRef::new(self.todo_storage.clone())
    }

    pub fn user(&self) -> ServiceUserRef {
        ServiceUserRef::new(self.user_storage.clone(), self.user_cache.clone())
    }

    pub fn auth(&self) -> ServiceAuthRef {
        ServiceAuthRef::new(self.session_storage.clone())
    }
}

impl Service {
    #[instrument(name = "Service::login_user", skip_all)]
    pub async fn login_user(
        &self,
        login: LoginUser,
        settings: &Settings,
    ) -> Result<LoginToken, AppError> {
        info!(email = %login.email, "login user");

        measure_and_record_service("login_user", || async {

            let user = self.user().get_by_email(&login.email).await.map_err(|_|AppError::UserByEmailNotFound)?;

            verify_password(&login.password, &user.hashed_password, &settings.auth)
            .await.map_err(|e| {
                    tracing::error!(error = ?e, "password verification failed");
                    e
                })?;

            let refresh_jti = Jti::new();
            let session = Session::new(&user.id, &refresh_jti, &settings.jwt)?;
            let session_id = session.id;
            let jwt_secret = std::env::var(JWT_SECRET_KEY).map_err(|_|AppError::FailedToLoadEnvVar(JWT_SECRET_KEY))?;

            self.session_storage.put(session.id, session).await?;

            let access_token = info_span!("generate_access_token").in_scope(|| {
                info!(user_id = %user.id, ttl_sec = settings.jwt.access_token_ttl_sec, "generating access token");
                trace_err!(jwt::generate_access_token(
                    user.id,
                    session_id,
                    &jwt_secret,
                    settings.jwt.access_token_ttl_sec,
                ), "failed to generate access token")
            })?;

            let refresh_token = info_span!("generate_refresh_token").in_scope(|| {
                info!(user_id = %user.id, ttl_days = settings.jwt.refresh_token_ttl_sec, "generating refresh token");
                trace_err!(jwt::generate_refresh_token(
                    user.id,
                    session_id,
                    refresh_jti,
                    &jwt_secret,
                    settings.jwt.refresh_token_ttl_sec,
                ), "failed to generate refresh token")
            })?;
            Ok(LoginToken{access_token, refresh_token})
        })
        .await
    }

    pub async fn flush_storage(&self) -> Result<(), AppError> {
        measure_and_record_service("flush_storage", || async {
            self.flush_storage.flush().await.map_err(Into::into)
        })
        .await
    }
}
