use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum ZhihuError {
    #[error("Missing access secret. Set ZHIHU_ACCESS_SECRET or run 'zhihu auth set-secret'.")]
    MissingSecret,

    #[error("Configuration directory could not be determined")]
    ConfigDirNotFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse config: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("Failed to serialize config: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("HTTP {status}: {body}")]
    Api { status: StatusCode, body: String },

    #[error("Non-JSON response from API")]
    NonJsonResponse,

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

pub type Result<T> = std::result::Result<T, ZhihuError>;
