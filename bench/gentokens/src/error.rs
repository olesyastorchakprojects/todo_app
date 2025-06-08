use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenTokensError {
    #[error("Failed to parse url from string")]
    UrlParse(#[from] url::ParseError),

    #[error("Failed to parse url from string")]
    Reqwest(#[from] reqwest::Error),

    #[error("Failed to read env vars")]
    ReadEnvVars(#[from] std::env::VarError),

    #[error("Failed to parse number from string")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Failed to parse json")]
    ParseJson(#[from] serde_json::Error),

    #[error("Failed to base64 decode")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("Io error")]
    Io(#[from] std::io::Error),
}
