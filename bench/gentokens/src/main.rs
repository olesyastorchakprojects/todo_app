use std::{io::Write, str::FromStr};

use error::GenTokensError;
use tokio::time::Instant;

mod client;
mod error;

#[derive(Debug, serde::Serialize)]
struct Token {
    token: String,
    email: String,
    password: String,
    id: String,
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: String,
}

#[tokio::main]
async fn main() -> Result<(), GenTokensError> {
    dotenv::dotenv().ok();

    let server_addr = std::env::var("ENV_SERVER_URL")?;
    let token_file = std::env::var("ENV_TOKEN_FILE")?;
    let token_count = std::env::var("ENV_TOKEN_COUNT")?.parse::<usize>()?;

    let url = url::Url::from_str(&server_addr)?;

    println!("Generating {} tokens", token_count);

    let start = Instant::now();

    generate(url, token_file, token_count).await?;

    let elapsed = start.elapsed();

    println!(
        "Generated {} tokens in {} ms",
        token_count,
        elapsed.as_millis()
    );

    Ok(())
}

async fn generate(
    url: url::Url,
    token_file: String,
    token_count: usize,
) -> Result<(), GenTokensError> {
    let client = client::Client::new(url).await;

    let admin_token = client
        .register_and_login("admin@gmail.com", "admin")
        .await
        .access_token;

    let mut tokens = Vec::with_capacity(token_count);
    for _ in 0..token_count {
        let email = random_string::generate(20, random_string::charsets::ALPHANUMERIC);
        let password = random_string::generate(20, random_string::charsets::ALPHANUMERIC);

        let token = client
            .register_and_login(&email, &password)
            .await
            .access_token;

        let response = client.get_user_by_email(&admin_token, &email).await;
        let user = response.json::<User>().await.unwrap();

        tokens.push(Token {
            token,
            email,
            password,
            id: user.id,
        });
    }

    let json = serde_json::to_string_pretty(&tokens)?;

    let mut file = std::fs::File::create(token_file)?;
    file.write_all(json.as_bytes())?;

    Ok(())
}
