use crate::error::{Result, ZhihuError};
use serde::{Deserialize, Serialize};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub access_secret: Option<String>,
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        dirs::home_dir()
            .map(|d| d.join(".zhihu-cli"))
            .ok_or(ZhihuError::ConfigDirNotFound)
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> Result<Config> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        let path = dir.join("config.toml");
        fs::create_dir_all(&dir)?;
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }
        Ok(())
    }

    pub fn resolve_secret() -> Result<String> {
        if let Ok(secret) = std::env::var("ZHIHU_ACCESS_SECRET") {
            let secret = secret.trim().to_string();
            if !secret.is_empty() {
                return Ok(secret);
            }
        }
        if let Some(secret) = Config::load()?.access_secret {
            let secret = secret.trim().to_string();
            if !secret.is_empty() {
                return Ok(secret);
            }
        }
        Err(ZhihuError::MissingSecret)
    }

    pub fn set_secret(secret: String) -> Result<()> {
        let mut config = Config::load()?;
        config.access_secret = Some(secret);
        config.save()
    }
}

/// Restore `HOME` to its prior value. Extracted from `with_temp_home` so the
/// `None => remove_var` branch is unit-testable in isolation. `#[cfg(test)]`
/// because it is only used by test helpers.
#[cfg(test)]
pub(crate) fn restore_home(original: Option<String>) {
    use std::env;
    unsafe {
        match original {
            Some(h) => env::set_var("HOME", h),
            None => env::remove_var("HOME"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir;

    fn with_temp_home<F>(f: F)
    where
        F: FnOnce(),
    {
        let tmp = TempDir::new().unwrap();
        let original = env::var("HOME").ok();
        unsafe {
            env::set_var("HOME", tmp.path());
            env::remove_var("ZHIHU_ACCESS_SECRET");
        }
        f();
        restore_home(original);
    }

    #[test]
    fn config_serde_roundtrip() {
        let config = Config {
            access_secret: Some("secret".into()),
        };
        let s = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&s).unwrap();
        assert_eq!(parsed.access_secret, Some("secret".into()));
    }

    #[test]
    fn empty_config_is_default() {
        let s = "";
        let config: Config = toml::from_str(s).unwrap();
        assert!(config.access_secret.is_none());
    }

    #[test]
    #[serial]
    fn resolve_secret_prefers_env_over_config() {
        with_temp_home(|| {
            Config::set_secret("from-config".into()).unwrap();
            unsafe { env::set_var("ZHIHU_ACCESS_SECRET", "from-env"); }
            assert_eq!(Config::resolve_secret().unwrap(), "from-env");
        });
    }

    #[test]
    #[serial]
    fn resolve_secret_returns_env_when_config_missing() {
        with_temp_home(|| {
            unsafe { env::set_var("ZHIHU_ACCESS_SECRET", "env-only"); }
            assert_eq!(Config::resolve_secret().unwrap(), "env-only");
            unsafe { env::remove_var("ZHIHU_ACCESS_SECRET"); }
        });
    }

    #[test]
    #[serial]
    fn resolve_secret_falls_back_to_config() {
        with_temp_home(|| {
            Config::set_secret("from-config".into()).unwrap();
            assert_eq!(Config::resolve_secret().unwrap(), "from-config");
        });
    }

    #[test]
    #[serial]
    fn resolve_secret_returns_config_when_env_unset() {
        with_temp_home(|| {
            Config::set_secret("cfg-only".into()).unwrap();
            // ZHIHU_ACCESS_SECRET explicitly removed by with_temp_home.
            assert_eq!(Config::resolve_secret().unwrap(), "cfg-only");
        });
    }

    #[test]
    #[serial]
    fn restore_home_unsets_when_original_was_none() {
        let original = env::var("HOME").ok();
        unsafe {
            env::set_var("HOME", "/tmp/non-existent-for-test");
        }
        restore_home(None);
        assert!(
            env::var("HOME").is_err(),
            "HOME should be unset after restore_home(None)"
        );
        // Restore so subsequent tests see the original environment.
        if let Some(o) = original {
            unsafe { env::set_var("HOME", o); }
        }
    }

    #[test]
    #[serial]
    fn resolve_secret_errors_when_missing() {
        with_temp_home(|| {
            let err = Config::resolve_secret().unwrap_err();
            assert!(matches!(err, ZhihuError::MissingSecret));
        });
    }

    #[test]
    #[serial]
    fn save_creates_config_file() {
        with_temp_home(|| {
            Config::set_secret("my-secret".into()).unwrap();
            let config = Config::load().unwrap();
            assert_eq!(config.access_secret, Some("my-secret".into()));
            assert!(Config::config_path().unwrap().exists());
        });
    }

    #[test]
    #[serial]
    #[cfg(unix)]
    fn config_file_has_restrictive_permissions() {
        with_temp_home(|| {
            Config::set_secret("my-secret".into()).unwrap();
            let path = Config::config_path().unwrap();
            let perms = fs::metadata(&path).unwrap().permissions();
            assert_eq!(perms.mode() & 0o777, 0o600);
        });
    }
}
