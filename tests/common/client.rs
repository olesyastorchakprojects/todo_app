#![allow(dead_code)]
use super::LoginResponse;
use reqwest::Url;
use todo_app::{TodoId, UserId};

pub struct TestAppClient {
    url: Url,
    client: reqwest::Client,
}

impl TestAppClient {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn register_user(&self, email: &str, password: &str) -> reqwest::Response {
        self.client
            .post(self.url.join("auth/register").unwrap())
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .send()
            .await
            .unwrap()
    }

    pub async fn login_user(&self, email: &str, password: &str) -> reqwest::Response {
        self.client
            .post(self.url.join("auth/login").unwrap())
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .send()
            .await
            .unwrap()
    }

    pub async fn register_and_login(&self, email: &str, password: &str) -> LoginResponse {
        self.client
            .post(self.url.join("auth/register").unwrap())
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .send()
            .await
            .unwrap();
        let res = self
            .client
            .post(self.url.join("auth/login").unwrap())
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .send()
            .await
            .unwrap();
        res.json::<LoginResponse>().await.unwrap()
    }

    pub async fn refresh_token(&self, token: &str) -> reqwest::Response {
        self.client
            .post(self.url.join("auth/refresh").unwrap())
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap()
    }

    pub async fn logout(&self, token: &str) -> reqwest::Response {
        self.client
            .post(self.url.join("auth/logout").unwrap())
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap()
    }

    pub async fn create_todo(&self, token: Option<&str>, text: Option<&str>) -> reqwest::Response {
        let request_builder = if let Some(token) = token {
            self.client
                .post(self.url.join("todos").unwrap())
                .header("Authorization", format!("Bearer {}", token))
                .json(&serde_json::json!({
                    "text": text.unwrap_or("aaa"),
                }))
        } else {
            self.client
                .post(self.url.join("todos").unwrap())
                .json(&serde_json::json!({
                    "text": text.unwrap_or("aaa"),
                }))
        };
        request_builder.send().await.unwrap()
    }

    pub async fn get_todo(&self, token: &str, todo_id: &str) -> reqwest::Response {
        self.client
            .get(self.url.join("todos/").unwrap().join(todo_id).unwrap())
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "text": "aaa",
            }))
            .send()
            .await
            .unwrap()
    }

    pub async fn get_all_todos(
        &self,
        token: &str,
        limit: usize,
        after: Option<TodoId>,
    ) -> reqwest::Response {
        let mut url = self.url.join("todos").unwrap();

        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("limit", &limit.to_string());

            if let Some(after) = after {
                query_pairs.append_pair("after", &after.to_string());
            }
        }

        self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap()
    }

    pub async fn update_todo(
        &self,
        token: &str,
        todo_id: &str,
        text: &str,
        group: &str,
    ) -> reqwest::Response {
        self.client
            .patch(self.url.join("todos/").unwrap().join(todo_id).unwrap())
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "text": text,
                "completed": true,
                "group": group
            }))
            .send()
            .await
            .unwrap()
    }

    pub async fn update_todo_with_empty_patch(
        &self,
        token: &str,
        todo_id: &str,
    ) -> reqwest::Response {
        self.client
            .patch(self.url.join("todos/").unwrap().join(todo_id).unwrap())
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap()
    }

    pub async fn delete_todo(&self, token: &str, todo_id: &str) -> reqwest::Response {
        self.client
            .delete(self.url.join("todos/").unwrap().join(todo_id).unwrap())
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap()
    }

    pub async fn get_all_users(
        &self,
        token: &str,
        limit: usize,
        after: Option<UserId>,
    ) -> reqwest::Response {
        let mut url = self.url.join("admin/users").unwrap();

        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("limit", &limit.to_string());

            if let Some(after) = after {
                query_pairs.append_pair("after", &after.to_string());
            }
        }

        self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap()
    }

    pub async fn promote_user(&self, token: &str, user_id: &UserId) -> reqwest::Response {
        let url = self
            .url
            .join(&format!("admin/user/{}/role", user_id))
            .unwrap();
        self.client
            .patch(url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "role": "admin",

            }))
            .send()
            .await
            .unwrap()
    }

    pub async fn get_user(&self, token: &str, user_id: &UserId) -> reqwest::Response {
        let url = self
            .url
            .join("/admin/user/")
            .unwrap()
            .join(&user_id.to_string())
            .unwrap();
        self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap()
    }

    pub async fn get_user_by_email(&self, token: &str, email: &str) -> reqwest::Response {
        let url = self
            .url
            .join("/admin/user/email/")
            .unwrap()
            .join(email)
            .unwrap();
        self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap()
    }
}
