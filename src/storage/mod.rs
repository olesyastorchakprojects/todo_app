mod error;
mod ids;
mod page;
mod session;
mod sled;
mod todo;
mod user;

#[cfg(feature = "integration_tests")]
pub use sled::test_util;
pub(crate) use sled::{error::SledStartupError, SledStorage};

use async_trait::async_trait;
pub(crate) use error::StorageError;
pub(crate) use page::Pagination;
pub use session::Session;
pub use todo::Todo;
pub(crate) use todo::{TodoVersion, UpdateTodo};
pub(crate) use user::Role;
pub use user::User;
pub(crate) use user::{HashedPassword, HASH_LEN, SALT_LEN};

pub use ids::{Jti, SessionId, TodoId, UserId};

#[async_trait]
pub trait TodoStorage: Send + Sync {
    async fn get(&self, user_id: UserId, id: TodoId) -> Result<Todo, StorageError>;
    async fn put(&self, user_id: UserId, id: TodoId, item: Todo) -> Result<(), StorageError>;
    async fn delete(&self, user_id: UserId, id: TodoId) -> Result<(), StorageError>;
    async fn update(
        &self,
        user_id: UserId,
        id: TodoId,
        patch: UpdateTodo,
    ) -> Result<(), StorageError>;

    async fn get_all(
        &self,
        user_id: UserId,
        page: Pagination<TodoId>,
    ) -> Result<(Vec<Todo>, Option<TodoId>), StorageError>;
    async fn delete_all(&self, user_id: UserId) -> Result<(), StorageError>;
}

#[async_trait]
pub trait UserStorage: Send + Sync {
    async fn get_by_email(&self, email: &str) -> Result<User, StorageError>;
    async fn get(&self, id: UserId) -> Result<User, StorageError>;
    async fn put(&self, id: UserId, user: User) -> Result<(), StorageError>;
    async fn delete(&self, id: UserId) -> Result<(), StorageError>;
    async fn update_role(&self, id: UserId, role: Role) -> Result<(), StorageError>;
    async fn get_all(
        &self,
        user_id: UserId,
        page: Pagination<UserId>,
    ) -> Result<(Vec<User>, Option<UserId>), StorageError>;
}

#[async_trait]
pub trait SessionStorage: Send + Sync {
    async fn get(&self, id: SessionId) -> Result<Session, StorageError>;
    async fn put(&self, id: SessionId, session: Session) -> Result<(), StorageError>;
    async fn delete(&self, id: SessionId) -> Result<(), StorageError>;
    async fn update(&self, id: SessionId, refresh_jti: Jti) -> Result<(), StorageError>;
}

#[async_trait]
pub trait FlushStorage: Send + Sync {
    async fn flush(&self) -> Result<(), StorageError>;
}
