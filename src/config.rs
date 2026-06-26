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

#[cfg(test)]
mod tests {
    use super::*;

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
}
