use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuickReplyError {
    #[error("Platform error ({platform}): {message}")]
    Platform { platform: String, message: String },

    #[error("Claude CLI error: {0}")]
    Claude(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Configuration error: {0}")]
    #[allow(dead_code)]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, QuickReplyError>;
