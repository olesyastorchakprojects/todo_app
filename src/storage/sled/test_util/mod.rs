#![allow(dead_code)]
use std::{path::PathBuf, sync::Arc};

use crate::{
    config::types::SledConfig,
    service::password::create_password_hash,
    storage::{
        sled::{
            BINCODE_CONFIG, SLED_EMAIL_TREE, SLED_SESSION_TREE, SLED_TODO_TREE, SLED_USER_TREE,
        },
        FlushStorage, Role, SessionStorage, Todo, TodoId, TodoStorage, User, UserId, UserStorage,
    },
    Settings,
};
use sled::Config;
use uuid::Uuid;

use super::SledStorage;

pub(crate) static ADMIN_UUID: Uuid = uuid::uuid!("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8");

pub struct TestStorageBuilder {
    todos: Vec<Todo>,
    users: Vec<User>,
    todo_storage: Arc<dyn TodoStorage>,
    user_storage: Arc<dyn UserStorage>,
    session_storage: Arc<dyn SessionStorage>,
    flush_storage: Arc<dyn FlushStorage>,
}

impl TestStorageBuilder {
    pub fn new() -> Self {
        let config = Config::new().temporary(true);
        let db = config.open().unwrap();
        let sled_storage = Arc::new(SledStorage {
            todo_tree: db.open_tree(SLED_TODO_TREE).unwrap(),
            user_tree: db.open_tree(SLED_USER_TREE).unwrap(),
            email_tree: db.open_tree(SLED_EMAIL_TREE).unwrap(),
            session_tree: db.open_tree(SLED_SESSION_TREE).unwrap(),
            bincode_config: BINCODE_CONFIG,
            storage_settings: SledConfig {
                path: PathBuf::from(""),
                delete_batch_size: 10,
            },
        });
        Self {
            todos: Vec::new(),
            users: Vec::new(),
            todo_storage: sled_storage.clone() as Arc<dyn TodoStorage>,
            user_storage: sled_storage.clone() as Arc<dyn UserStorage>,
            session_storage: sled_storage.clone() as Arc<dyn SessionStorage>,
            flush_storage: sled_storage.clone() as Arc<dyn FlushStorage>,
        }
    }

    pub fn with_todos(mut self, count: usize) -> Self {
        self.todos = (0..count)
            .map(|i| Todo {
                id: TodoId::new(),
                text: format!("todo {}", i),
                completed: false,
                group: String::from("group"),
            })
            .collect();
        self
    }

    pub async fn with_users(mut self, count: usize) -> Self {
        let hash = create_password_hash("password", &test_settings().auth)
            .await
            .unwrap();
        self.users = (0..count)
            .map(|_| User {
                id: UserId::new(),
                email: String::new(),
                hashed_password: hash.clone(),
                role: Role::User,
            })
            .collect();
        self
    }

    pub async fn build_todo(&self) -> Arc<dyn TodoStorage> {
        for todo in &self.todos {
            self.todo_storage
                .put(ADMIN_UUID.into(), todo.id, todo.clone())
                .await
                .unwrap();
        }

        self.todo_storage.clone()
    }

    pub async fn build_session(&self) -> Arc<dyn SessionStorage> {
        self.session_storage.clone()
    }

    pub async fn build_flush(&self) -> Arc<dyn FlushStorage> {
        self.flush_storage.clone()
    }

    pub async fn build_user(&self) -> Arc<dyn UserStorage> {
        for user in &self.users {
            self.user_storage.put(user.id, user.clone()).await.unwrap();
        }

        self.user_storage.clone()
    }

    pub fn todos(&self) -> Vec<Todo> {
        self.todos.clone()
    }
}

impl Default for TestStorageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn test_settings() -> Settings {
    Settings::from_file("test").unwrap()
}
