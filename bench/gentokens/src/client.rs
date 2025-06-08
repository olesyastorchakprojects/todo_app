use reqwest::{Method, Url};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

#[derive(Debug, serde::Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
}

pub struct Client {
    url: Url,
    client: ClientWithMiddleware,
}

impl Client {
    pub async fn new(url: Url) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Self { url, client }
    }

    pub async fn register_and_login(&self, email: &str, password: &str) -> LoginResponse {
        let url = self.url.join("auth/register").unwrap();
        let request = reqwest::Client::new()
            .request(Method::POST, url)
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .build()
            .unwrap();

        let _response = self.client.execute(request).await.unwrap();

        let url = self.url.join("auth/login").unwrap();
        let request = reqwest::Client::new()
            .request(Method::POST, url)
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .build()
            .unwrap();

        let res = self.client.execute(request).await.unwrap();
        res.json::<LoginResponse>().await.unwrap()
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
