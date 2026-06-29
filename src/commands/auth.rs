use crate::cli::AuthCommand;
use crate::config::Config;
use crate::error::{Result, ZhihuError};
use crate::output::{dispatch_result, print_error};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

pub async fn run(cmd: AuthCommand) {
    let stdin = io::stdin();
    let mut lock = stdin.lock();
    if let Err(e) = dispatch_result(handle(cmd, &mut lock)) {
        print_error(&e);
    }
}

fn handle(cmd: AuthCommand, stdin: &mut impl BufRead) -> Result<Value> {
    match cmd {
        AuthCommand::Login => {
            print!("Enter access secret: ");
            io::stdout().flush().unwrap();
            let secret = read_and_validate_secret(stdin)?;
            save_secret(secret)
        }
        AuthCommand::SetSecret { secret } => {
            let trimmed = secret.trim().to_string();
            validate_secret(&trimmed)?;
            save_secret(trimmed)
        }
        AuthCommand::Status => {
            let config = Config::load()?;
            let payload = status_payload(
                std::env::var("ZHIHU_ACCESS_SECRET").is_ok(),
                config.access_secret.is_some(),
            );
            Ok(payload)
        }
    }
}

/// Persist a validated secret to the config file and return the success JSON.
pub(crate) fn save_secret(secret: String) -> Result<Value> {
    Config::set_secret(secret)?;
    Ok(json!({"status":"ok","message":"secret saved"}))
}

/// Read a single line from `reader`, trim it, and reject empty/whitespace-only
/// input. Centralizes the validation so both `Login` (stdin) and `SetSecret`
/// (CLI arg) share the same rules.
pub(crate) fn read_and_validate_secret(reader: &mut impl BufRead) -> Result<String> {
    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    validate_secret(buf.trim())
}

/// Trim a candidate secret and reject it if empty. Pure function so the
/// validation rule can be unit-tested without any I/O.
pub(crate) fn validate_secret(secret: &str) -> Result<String> {
    let trimmed = secret.trim();
    if trimmed.is_empty() {
        return Err(ZhihuError::InvalidArgument(
            "secret cannot be empty".into(),
        ));
    }
    Ok(trimmed.to_string())
}

/// Compute the JSON payload for the `auth status` command given the two
/// inputs that drive its output: whether `ZHIHU_ACCESS_SECRET` is set, and
/// whether the config file holds a secret. Environment takes precedence.
pub(crate) fn status_payload(env_set: bool, config_set: bool) -> Value {
    json!({
        "configured": env_set || config_set,
        "source": if env_set {
            "env"
        } else if config_set {
            "config"
        } else {
            "none"
        },
    })
}

#[cfg(test)]
mod tests {
    //! Unit tests for the auth command's pure logic. The I/O layers (real
    //! stdin, real config file) are exercised by `tests/cli.rs::auth_*`.

    use super::{handle, read_and_validate_secret, save_secret, status_payload, validate_secret};
    use crate::cli::AuthCommand;
    use crate::config::Config;
    use crate::error::ZhihuError;
    use serde_json::json;
    use serial_test::serial;
    use std::env;
    use std::io;

    // ---- save_secret ----

    #[test]
    #[serial]
    fn save_secret_persists_and_returns_success() {
        restore_home_via_with_temp_home(|| {
            let value = save_secret("my-key".into()).expect("save_secret should succeed");
            assert_eq!(value, json!({"status":"ok","message":"secret saved"}));
            let loaded = Config::load().unwrap();
            assert_eq!(loaded.access_secret, Some("my-key".into()));
        });
    }

    fn restore_home_via_with_temp_home<F: FnOnce()>(f: F) {
        let tmp = tempfile::TempDir::new().unwrap();
        let original = env::var("HOME").ok();
        unsafe {
            env::set_var("HOME", tmp.path());
            env::remove_var("ZHIHU_ACCESS_SECRET");
        }
        f();
        unsafe {
            match original {
                Some(h) => env::set_var("HOME", h),
                None => env::remove_var("HOME"),
            }
        }
    }

    // ---- handle ----

    #[test]
    #[serial]
    fn handle_set_secret_persists_to_config() {
        restore_home_via_with_temp_home(|| {
            let result = handle(
                AuthCommand::SetSecret {
                    secret: "from-arg".into(),
                },
                &mut io::empty(),
            )
            .expect("SetSecret should succeed");
            assert_eq!(result, json!({"status":"ok","message":"secret saved"}));
            let loaded = Config::load().unwrap();
            assert_eq!(loaded.access_secret, Some("from-arg".into()));
        });
    }

    #[test]
    #[serial]
    fn handle_set_secret_rejects_empty() {
        let result = handle(
            AuthCommand::SetSecret { secret: "".into() },
            &mut io::empty(),
        );
        assert!(matches!(result, Err(ZhihuError::InvalidArgument(_))));
    }

    #[test]
    #[serial]
    fn handle_status_returns_configured_when_config_has_secret() {
        restore_home_via_with_temp_home(|| {
            Config::set_secret("x".into()).unwrap();
            let result = handle(AuthCommand::Status, &mut io::empty()).expect("status should succeed");
            assert_eq!(result["configured"], true);
            assert_eq!(result["source"], "config");
        });
    }

    #[test]
    #[serial]
    fn handle_login_reads_secret_from_stdin() {
        restore_home_via_with_temp_home(|| {
            let mut input: &[u8] = b"stdin-secret\n";
            let result =
                handle(AuthCommand::Login, &mut input).expect("login should succeed");
            assert_eq!(result, json!({"status":"ok","message":"secret saved"}));
            let loaded = Config::load().unwrap();
            assert_eq!(loaded.access_secret, Some("stdin-secret".into()));
        });
    }

    #[test]
    #[serial]
    fn handle_login_rejects_empty_stdin() {
        let mut input: &[u8] = b"\n";
        let result = handle(AuthCommand::Login, &mut input);
        assert!(matches!(result, Err(ZhihuError::InvalidArgument(_))));
    }

    // ---- validate_secret ----

    // 1. Empty string is rejected with the exact message the CLI uses.
    #[test]
    fn validate_secret_rejects_empty() {
        let err = validate_secret("").expect_err("empty must be rejected");
        match err {
            ZhihuError::InvalidArgument(msg) => assert_eq!(msg, "secret cannot be empty"),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    // 2. Whitespace-only input is rejected.
    #[test]
    fn validate_secret_rejects_whitespace_only() {
        assert!(validate_secret("   ").is_err());
        assert!(validate_secret("\t").is_err());
        assert!(validate_secret("\n").is_err());
        assert!(validate_secret("  \t \n ").is_err());
    }

    // 3. Valid input is returned (without modification if already trimmed).
    #[test]
    fn validate_secret_accepts_non_empty() {
        assert_eq!(validate_secret("abc").unwrap(), "abc");
        assert_eq!(validate_secret("中文").unwrap(), "中文");
    }

    // 4. Input with surrounding whitespace gets trimmed.
    #[test]
    fn validate_secret_trims_surrounding_whitespace() {
        assert_eq!(validate_secret("  abc  ").unwrap(), "abc");
        assert_eq!(validate_secret("\nabc\n").unwrap(), "abc");
    }

    // ---- read_and_validate_secret ----

    // 5. A well-formed line is returned trimmed, without the trailing newline.
    #[test]
    fn read_and_validate_secret_returns_trimmed_line() {
        let mut input: &[u8] = b"my-secret\n";
        assert_eq!(
            read_and_validate_secret(&mut input).unwrap(),
            "my-secret"
        );
    }

    // 6. Surrounding whitespace on the line is trimmed.
    #[test]
    fn read_and_validate_secret_trims_surrounding_whitespace() {
        let mut input: &[u8] = b"  spaced  \n";
        assert_eq!(read_and_validate_secret(&mut input).unwrap(), "spaced");
    }

    // 7. Empty stdin (just a newline) is rejected.
    #[test]
    fn read_and_validate_secret_rejects_empty_line() {
        let mut input: &[u8] = b"\n";
        let err = read_and_validate_secret(&mut input).expect_err("newline-only must be rejected");
        match err {
            ZhihuError::InvalidArgument(msg) => assert_eq!(msg, "secret cannot be empty"),
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    // 8. Whitespace-only line is rejected.
    #[test]
    fn read_and_validate_secret_rejects_whitespace_only_line() {
        let mut input: &[u8] = b"   \t  \n";
        assert!(read_and_validate_secret(&mut input).is_err());
    }

    // ---- status_payload ----

    // 9. Neither env nor config: not configured, source "none".
    #[test]
    fn status_payload_neither_set() {
        assert_eq!(
            status_payload(false, false),
            json!({"configured": false, "source": "none"})
        );
    }

    // 10. Only env set: configured, source "env".
    #[test]
    fn status_payload_only_env_set() {
        assert_eq!(
            status_payload(true, false),
            json!({"configured": true, "source": "env"})
        );
    }

    // 11. Only config set: configured, source "config".
    #[test]
    fn status_payload_only_config_set() {
        assert_eq!(
            status_payload(false, true),
            json!({"configured": true, "source": "config"})
        );
    }

    // 12. Both set: env wins (precedence rule).
    #[test]
    fn status_payload_both_set_env_wins() {
        assert_eq!(
            status_payload(true, true),
            json!({"configured": true, "source": "env"})
        );
    }
}
