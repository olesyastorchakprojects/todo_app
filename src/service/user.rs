use std::sync::Arc;

use tracing::{info, instrument};

use crate::{
    handlers::{error::AppError, DisplayUser, RegisterUser},
    storage::{Pagination, Role, User, UserId, UserStorage},
    utils::measure_metrics::measure_and_record_service,
    Settings,
};

use super::{password::create_password_hash, UserCache};

pub struct ServiceUserRef {
    storage: Arc<dyn UserStorage>,
    user_cache: Arc<UserCache>,
}

impl ServiceUserRef {
    pub(crate) fn new(storage: Arc<dyn UserStorage>, user_cache: Arc<UserCache>) -> Self {
        Self {
            storage,
            user_cache,
        }
    }

    #[instrument(name = "Service::user::create_admins", skip(self))]
    pub async fn create_admins(&self, settings: &Settings) -> Result<(), AppError> {
        let result = measure_and_record_service("register_user", || async {
            for admin in &settings.auth.admins {
                self.create_new_user(
                    RegisterUser {
                        email: admin.email.clone(),
                        password: admin.password.clone(),
                    },
                    Role::Admin,
                    settings,
                )
                .await?;
            }
            Ok(())
        })
        .await;
        if let Err(AppError::UserAlreadyExists) = result {
            Ok(())
        } else {
            Ok(result?)
        }
    }

    #[instrument(name = "Service::user::add", skip_all)]
    pub(crate) async fn add(
        &self,
        new_user: RegisterUser,
        settings: &Settings,
    ) -> Result<(), AppError> {
        info!(email = %new_user.email, "register new user");

        measure_and_record_service("register_user", || async {
            self.create_new_user(new_user, Role::User, settings).await
        })
        .await
    }

    #[instrument(name = "Service::user::get", skip_all)]
    pub(crate) async fn get(&self, id: UserId) -> Result<User, AppError> {
        info!(user_id = %id, "get user by id");

        if let Some(user) = self.user_cache.by_id.get(&id).await {
            info!(user_id = %id, "found user by id in cache");
            return Ok(user.clone());
        }

        let result =
            measure_and_record_service("get_user_by_id", || async { self.storage.get(id).await })
                .await;

        if let Ok(user) = &result {
            if self.user_cache.by_id.get(&id).await.is_none() {
                self.insert_user_into_cache(user).await;
            }
        }

        Ok(result?)
    }

    #[instrument(name = "Service::user::get_by_email", skip_all)]
    pub(crate) async fn get_by_email(&self, email: &str) -> Result<User, AppError> {
        info!(email = %email, "get user by email");

        if let Some(user) = self.user_cache.by_email.get(email).await {
            info!(user_email = %email, "found user by email in cache");
            return Ok(user.clone());
        }

        let result = measure_and_record_service("get_user_by_email", || async {
            self.storage.get_by_email(email).await
        })
        .await;

        if let Ok(user) = &result {
            if self.user_cache.by_email.get(email).await.is_none() {
                self.insert_user_into_cache(user).await;
            }
        }

        Ok(result?)
    }

    #[instrument(name = "Service::user::get_all", skip_all, fields(after_is_some = page.after.is_some(),
    limit = page.limit))]
    pub(crate) async fn get_all(
        &self,
        user: &User,
        page: Pagination<UserId>,
    ) -> Result<(Vec<DisplayUser>, Option<UserId>), AppError> {
        info!(page_after = ?page.after, "get all todos with page");

        let (users, cursor) = measure_and_record_service("get_users", || async {
            self.storage.get_all(user.id, page).await
        })
        .await?;

        Ok((users.into_iter().map(Into::into).collect(), cursor))
    }

    #[instrument(name = "Service::user::update", skip_all, fields(role = ?role))]
    pub(crate) async fn update(
        &self,
        user: &User,
        update_user_id: UserId,
        role: Role,
    ) -> Result<(), AppError> {
        info!(update_user_id = %update_user_id, "update user");

        if user.id == update_user_id {
            return Err(AppError::Forbidden);
        }

        let result = measure_and_record_service("change_user_role", || async {
            let result = self.storage.update_role(update_user_id, role).await;
            if result.is_ok() {
                if let Some(user) = self.user_cache.by_id.get(&update_user_id).await {
                    self.invalidate_user_in_cache(&user).await;
                }
            }
            result
        })
        .await;

        Ok(result?)
    }

    #[instrument(name = "Service::user::delete", skip_all)]
    pub(crate) async fn delete(&self, user: &User, delete_user_id: UserId) -> Result<(), AppError> {
        info!(delete_user_id = %delete_user_id, "delete user");

        if user.id == delete_user_id {
            return Err(AppError::Forbidden);
        }

        let result = measure_and_record_service("delete_user", || async {
            let result = self.storage.delete(delete_user_id).await;
            if result.is_ok() {
                if let Some(user) = self.user_cache.by_id.get(&delete_user_id).await {
                    self.invalidate_user_in_cache(&user).await;
                }
            }
            result
        })
        .await;

        Ok(result?)
    }
}

impl ServiceUserRef {
    async fn insert_user_into_cache(&self, user: &User) {
        self.user_cache.by_id.insert(user.id, user.clone()).await;
        self.user_cache
            .by_email
            .insert(user.email.clone(), user.clone())
            .await;
    }

    async fn invalidate_user_in_cache(&self, user: &User) {
        self.user_cache.by_email.invalidate(&user.email).await;
        self.user_cache.by_id.invalidate(&user.id).await;
    }

    #[instrument(name = "Service::user::create_new_user", skip_all, fields(role = ?role))]
    async fn create_new_user(
        &self,
        new_user: RegisterUser,
        role: Role,
        settings: &Settings,
    ) -> Result<(), AppError> {
        measure_and_record_service("register_user", || async {
            if self
                .user_cache
                .by_email
                .get(&new_user.email)
                .await
                .is_some()
            {
                info!(user_email = %new_user.email, "found user by email in cache");
                return Err(AppError::UserAlreadyExists);
            }

            if self
                .storage
                .get_by_email(new_user.email.as_ref())
                .await
                .is_ok()
            {
                return Err(AppError::UserAlreadyExists);
            }

            let hashed_password = create_password_hash(&new_user.password, &settings.auth).await?;

            let id = UserId::new();
            let user = User {
                id,
                email: new_user.email,
                hashed_password,
                role,
            };

            self.storage
                .put(id, user.clone())
                .await
                .map_err(AppError::from)
        })
        .await
    }
}
