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

#[cfg(test)]
mod tests {
    //! Display tests for `ZhihuError` variants.
    //!
    //! These lock in the exact strings the user-facing CLI emits on stderr.
    //! If anyone changes a `#[error("...")]` attribute in the enum above,
    //! the affected test fails — preserving CLI behavior.
    //!
    //! Note: this is **test-after** (the impls already exist). It documents
    //! the contract and prevents regression; it does not drive new design.

    use super::ZhihuError;
    use reqwest::StatusCode;

    #[test]
    fn missing_secret_displays_setup_hint() {
        let err = ZhihuError::MissingSecret;
        assert_eq!(
            err.to_string(),
            "Missing access secret. Set ZHIHU_ACCESS_SECRET or run 'zhihu auth set-secret'."
        );
    }

    #[test]
    fn config_dir_not_found_displays_generic_message() {
        let err = ZhihuError::ConfigDirNotFound;
        assert_eq!(
            err.to_string(),
            "Configuration directory could not be determined"
        );
    }

    #[test]
    fn io_error_wraps_inner_message() {
        let inner = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = ZhihuError::Io(inner);
        assert_eq!(err.to_string(), "IO error: file missing");
    }

    #[test]
    fn config_parse_error_wraps_inner_message() {
        let inner: toml::de::Error = toml::from_str::<toml::Value>("not = valid = toml")
            .expect_err("should produce a parse error");
        let err = ZhihuError::ConfigParse(inner);
        let rendered = err.to_string();
        assert!(
            rendered.starts_with("Failed to parse config: "),
            "expected wrapped prefix, got: {rendered:?}"
        );
    }

    #[test]
    fn config_serialize_error_wraps_inner_message() {
        // NaN cannot be serialized to TOML — guarantees a ser::Error.
        let bad = toml::Value::Float(f64::NAN);
        let inner: toml::ser::Error = toml::to_string(&bad).unwrap_err();
        let err = ZhihuError::ConfigSerialize(inner);
        let rendered = err.to_string();
        assert!(
            rendered.starts_with("Failed to serialize config: "),
            "expected wrapped prefix, got: {rendered:?}"
        );
    }

    #[test]
    fn http_error_wraps_inner_message() {
        // Constructing a request with an invalid URL produces reqwest::Error.
        let inner: reqwest::Error = reqwest::Client::new()
            .get("not a url")
            .build()
            .unwrap_err();
        let err = ZhihuError::Http(inner);
        let rendered = err.to_string();
        assert!(
            rendered.starts_with("HTTP request failed: "),
            "expected wrapped prefix, got: {rendered:?}"
        );
    }

    #[test]
    fn api_error_includes_status_and_body() {
        let err = ZhihuError::Api {
            status: StatusCode::NOT_FOUND,
            body: "{\"error\":\"not found\"}".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "HTTP 404 Not Found: {\"error\":\"not found\"}"
        );
    }

    #[test]
    fn non_json_response_displays_clear_message() {
        let err = ZhihuError::NonJsonResponse;
        assert_eq!(err.to_string(), "Non-JSON response from API");
    }

    #[test]
    fn invalid_argument_includes_user_supplied_reason() {
        let err = ZhihuError::InvalidArgument("secret cannot be empty".to_string());
        assert_eq!(err.to_string(), "Invalid argument: secret cannot be empty");
    }
}
