use assert_cmd::Command;
use predicates::prelude::*;
use std::env;
use tempfile::TempDir;

fn with_temp_home<F>(f: F)
where
    F: FnOnce(&TempDir),
{
    let tmp = TempDir::new().unwrap();
    // Ensure tests don't inherit a real access secret from the environment.
    unsafe { env::remove_var("ZHIHU_ACCESS_SECRET"); }
    f(&tmp);
}

#[test]
fn help_shows_commands() {
    let mut cmd = Command::cargo_bin("zhihu").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("auth"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("ask"));
}

#[test]
fn auth_status_unconfigured() {
    with_temp_home(|tmp| {
        let mut cmd = Command::cargo_bin("zhihu").unwrap();
        cmd.env("HOME", tmp.path());
        cmd.arg("auth").arg("status");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("\"configured\": false"))
            .stdout(predicate::str::contains("\"source\": \"none\""));
    });
}

#[test]
fn auth_set_secret_and_status() {
    with_temp_home(|tmp| {
        let mut cmd = Command::cargo_bin("zhihu").unwrap();
        cmd.env("HOME", tmp.path());
        cmd.arg("auth").arg("set-secret").arg("my-secret");
        cmd.assert().success();

        let mut cmd = Command::cargo_bin("zhihu").unwrap();
        cmd.env("HOME", tmp.path());
        cmd.arg("auth").arg("status");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("\"configured\": true"))
            .stdout(predicate::str::contains("\"source\": \"config\""));
    });
}

#[test]
fn search_zhihu_without_auth_fails() {
    with_temp_home(|tmp| {
        let mut cmd = Command::cargo_bin("zhihu").unwrap();
        cmd.env("HOME", tmp.path());
        cmd.env_remove("ZHIHU_ACCESS_SECRET");
        cmd.arg("search").arg("zhihu").arg("query");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("\"code\":20001"))
            .stderr(predicate::str::contains("Missing access secret"));
    });
}

#[test]
fn env_secret_overrides_config() {
    with_temp_home(|tmp| {
        let mut cmd = Command::cargo_bin("zhihu").unwrap();
        cmd.env("HOME", tmp.path());
        cmd.arg("auth").arg("set-secret").arg("config-secret");
        cmd.assert().success();

        let mut cmd = Command::cargo_bin("zhihu").unwrap();
        cmd.env("HOME", tmp.path());
        cmd.env("ZHIHU_ACCESS_SECRET", "env-secret");
        cmd.arg("auth").arg("status");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("\"configured\": true"))
            .stdout(predicate::str::contains("\"source\": \"env\""));
    });
}
