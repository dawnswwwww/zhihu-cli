use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use serial_test::serial;
use std::env;
use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

#[test]
fn auth_set_secret_empty_value_fails() {
    with_temp_home(|tmp| {
        let mut cmd = Command::cargo_bin("zhihu").unwrap();
        cmd.env("HOME", tmp.path());
        cmd.arg("auth").arg("set-secret").arg("");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("secret cannot be empty"));
    });
}

// ---- Wiremock-backed tests: exercise the full `run` body of search/ask ----

#[tokio::test]
#[serial]
async fn cli_search_zhihu_against_mock_server_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/content/zhihu_search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Code": 0,
            "Message": "ok",
            "Data": [{"title": "stub"}],
        })))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("zhihu").unwrap();
    cmd.env("ZHIHU_ACCESS_SECRET", "fake");
    cmd.env("ZHIHU_OPENAPI_BASE_URL", server.uri());
    cmd.arg("search").arg("zhihu").arg("rust");
    cmd.assert().success();
}

#[tokio::test]
#[serial]
async fn cli_ask_non_stream_against_mock_server_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "stub",
            "choices": [{"message": {"role": "assistant", "content": "hi"}}],
        })))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("zhihu").unwrap();
    cmd.env("ZHIHU_ACCESS_SECRET", "fake");
    cmd.env("ZHIHU_OPENAPI_BASE_URL", server.uri());
    cmd.arg("ask").arg("hello");
    cmd.assert().success();
}

#[tokio::test]
#[serial]
async fn cli_ask_stream_against_mock_server_completes() {
    let sse_body = concat!(
        "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"}}]}\n",
        "\n",
        "data: [DONE]\n",
        "\n",
    );
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(sse_body))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("zhihu").unwrap();
    cmd.env("ZHIHU_ACCESS_SECRET", "fake");
    cmd.env("ZHIHU_OPENAPI_BASE_URL", server.uri());
    cmd.arg("ask").arg("hello").arg("--stream");
    cmd.assert().success();
}
