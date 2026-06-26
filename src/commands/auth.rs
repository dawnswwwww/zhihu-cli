use crate::cli::AuthCommand;
use crate::config::Config;
use crate::error::Result;
use crate::output::{print_error, print_json};
use serde_json::json;
use std::io::{self, Write};

pub async fn run(cmd: AuthCommand) {
    match handle(cmd).await {
        Ok(value) => print_json(&value),
        Err(e) => print_error(&e),
    }
}

async fn handle(cmd: AuthCommand) -> Result<serde_json::Value> {
    match cmd {
        AuthCommand::Login => {
            print!("Enter access secret: ");
            io::stdout().flush().unwrap();
            let mut secret = String::new();
            io::stdin().read_line(&mut secret)?;
            let secret = secret.trim().to_string();
            if secret.is_empty() {
                return Err(crate::error::ZhihuError::InvalidArgument(
                    "secret cannot be empty".into(),
                ));
            }
            Config::set_secret(secret)?;
            Ok(json!({"status":"ok","message":"secret saved"}))
        }
        AuthCommand::SetSecret { secret } => {
            let secret = secret.trim().to_string();
            if secret.is_empty() {
                return Err(crate::error::ZhihuError::InvalidArgument(
                    "secret cannot be empty".into(),
                ));
            }
            Config::set_secret(secret)?;
            Ok(json!({"status":"ok","message":"secret saved"}))
        }
        AuthCommand::Status => {
            let config = Config::load()?;
            let configured = config.access_secret.is_some()
                || std::env::var("ZHIHU_ACCESS_SECRET").is_ok();
            let source = if std::env::var("ZHIHU_ACCESS_SECRET").is_ok() {
                "env"
            } else if config.access_secret.is_some() {
                "config"
            } else {
                "none"
            };
            Ok(json!({
                "configured": configured,
                "source": source,
            }))
        }
    }
}
