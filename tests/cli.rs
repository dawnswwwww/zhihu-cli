use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use serial_test::serial;
use std::env;
use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
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
        .stdout(predicate::str::contains("ask"))
        .stdout(predicate::str::contains("hot"))
        .stdout(predicate::str::contains("quota"))
        .stdout(predicate::str::contains("user"))
        .stdout(predicate::str::contains("kb"))
        .stdout(predicate::str::contains("pdf"))
        .stdout(predicate::str::contains("ppt"));
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
fn hot_without_auth_fails() {
    with_temp_home(|tmp| {
        let mut cmd = Command::cargo_bin("zhihu").unwrap();
        cmd.env("HOME", tmp.path());
        cmd.env_remove("ZHIHU_ACCESS_SECRET");
        cmd.arg("hot");
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

#[tokio::test]
#[serial]
async fn cli_hot_against_mock_server_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/content/hot_list"))
        .and(query_param("Limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Code": 0,
            "Message": "ok",
            "Data": { "Total": 0, "Items": [] },
        })))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("zhihu").unwrap();
    cmd.env("ZHIHU_ACCESS_SECRET", "fake");
    cmd.env("ZHIHU_OPENAPI_BASE_URL", server.uri());
    cmd.arg("hot").arg("--limit").arg("5");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"Code\": 0"));
}

#[test]
fn quota_without_auth_fails() {
    with_temp_home(|tmp| {
        let mut cmd = Command::cargo_bin("zhihu").unwrap();
        cmd.env("HOME", tmp.path());
        cmd.env_remove("ZHIHU_ACCESS_SECRET");
        cmd.arg("quota");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("\"code\":20001"))
            .stderr(predicate::str::contains("Missing access secret"));
    });
}

#[tokio::test]
#[serial]
async fn cli_quota_against_mock_server_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/quota"))
        .and(query_param("APIIDs", "knowledge"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Code": 0,
            "Message": "success",
            "Data": [{ "APIID": "knowledge", "TotalQuota": 500, "TotalUsed": 12, "RemainingQuota": 488 }],
        })))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("zhihu").unwrap();
    cmd.env("ZHIHU_ACCESS_SECRET", "fake");
    cmd.env("ZHIHU_OPENAPI_BASE_URL", server.uri());
    cmd.arg("quota").arg("--ids").arg("knowledge");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"Code\": 0"));
}

#[test]
fn user_without_auth_fails() {
    with_temp_home(|tmp| {
        let mut cmd = Command::cargo_bin("zhihu").unwrap();
        cmd.env("HOME", tmp.path());
        cmd.env_remove("ZHIHU_ACCESS_SECRET");
        cmd.arg("user").arg("favlists");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("\"code\":20001"))
            .stderr(predicate::str::contains("Missing access secret"));
    });
}

#[tokio::test]
#[serial]
async fn cli_user_contents_against_mock_server_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/user/contents"))
        .and(query_param("ContentType", "answer"))
        .and(query_param("Limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Code": 0,
            "Message": "success",
            "Data": { "Items": [], "Paging": { "IsEnd": true, "Totals": 0 } },
        })))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("zhihu").unwrap();
    cmd.env("ZHIHU_ACCESS_SECRET", "fake");
    cmd.env("ZHIHU_OPENAPI_BASE_URL", server.uri());
    cmd.arg("user").arg("contents").arg("--type").arg("answer").arg("--limit").arg("5");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"Code\": 0"));
}

#[test]
fn kb_without_auth_fails() {
    with_temp_home(|tmp| {
        let mut cmd = Command::cargo_bin("zhihu").unwrap();
        cmd.env("HOME", tmp.path());
        cmd.env_remove("ZHIHU_ACCESS_SECRET");
        cmd.arg("kb").arg("list");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("\"code\":20001"))
            .stderr(predicate::str::contains("Missing access secret"));
    });
}

#[tokio::test]
#[serial]
async fn cli_kb_list_against_mock_server_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/knowledge/bases"))
        .and(query_param("Scope", "all"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Code": 0,
            "Message": "success",
            "Data": { "Items": [] },
        })))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("zhihu").unwrap();
    cmd.env("ZHIHU_ACCESS_SECRET", "fake");
    cmd.env("ZHIHU_OPENAPI_BASE_URL", server.uri());
    cmd.arg("kb").arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"Code\": 0"));
}

#[test]
fn kb_search_without_ids_or_scopes_fails() {
    with_temp_home(|tmp| {
        let mut cmd = Command::cargo_bin("zhihu").unwrap();
        cmd.env("HOME", tmp.path());
        cmd.env("ZHIHU_ACCESS_SECRET", "fake");
        cmd.arg("kb").arg("search").arg("query");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Invalid argument"))
            .stderr(predicate::str::contains("at least one --kb-id or --scope"));
    });
}

#[tokio::test]
#[serial]
async fn cli_kb_upload_against_mock_server_succeeds() {
    let tmp = tempfile::TempDir::new().unwrap();
    let file_path = tmp.path().join("doc.md");
    std::fs::write(&file_path, "# hello kb").unwrap();

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/knowledge/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Code": 0,
            "Message": "success",
            "Data": { "KnowledgeBaseID": "kb-1", "RecallContentID": "abc", "FileName": "doc.md" },
        })))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("zhihu").unwrap();
    cmd.env("ZHIHU_ACCESS_SECRET", "fake");
    cmd.env("ZHIHU_OPENAPI_BASE_URL", server.uri());
    cmd.arg("kb").arg("upload").arg(file_path.to_str().unwrap());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"Code\": 0"));
}

#[test]
fn pdf_without_auth_fails() {
    with_temp_home(|tmp| {
        let mut cmd = Command::cargo_bin("zhihu").unwrap();
        cmd.env("HOME", tmp.path());
        cmd.env_remove("ZHIHU_ACCESS_SECRET");
        cmd.arg("pdf").arg("status").arg("pdf_1");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("\"code\":20001"))
            .stderr(predicate::str::contains("Missing access secret"));
    });
}

#[tokio::test]
#[serial]
async fn cli_pdf_parse_one_shot_against_mock_server_succeeds() {
    let tmp = tempfile::TempDir::new().unwrap();
    let file_path = tmp.path().join("report.pdf");
    std::fs::write(&file_path, "%PDF-1.4 fake").unwrap();

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/pdf-parse/tasks/pdf_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Code": 0, "Message": "success",
            "Data": {
                "task_id": "pdf_1", "task_status": "succeeded", "progress": 1,
                "result": { "url": "https://example.com/result.json", "expires_at_ms": 1782800000000_i64 }
            }
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/pdf-parse/tasks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Code": 0, "Message": "success",
            "Data": { "task_id": "pdf_1", "task_status": "pending" }
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/resources/v1/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Code": 0, "Message": "success", "Data": { "file_id": "file_abc" }
        })))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("zhihu").unwrap();
    cmd.env("ZHIHU_ACCESS_SECRET", "fake");
    cmd.env("ZHIHU_OPENAPI_BASE_URL", server.uri());
    cmd.arg("pdf")
        .arg("parse")
        .arg(file_path.to_str().unwrap())
        .arg("--timeout-secs")
        .arg("30");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"task_status\": \"succeeded\""))
        .stdout(predicate::str::contains("result.json"));
}

#[test]
fn ppt_without_auth_fails() {
    with_temp_home(|tmp| {
        let mut cmd = Command::cargo_bin("zhihu").unwrap();
        cmd.env("HOME", tmp.path());
        cmd.env_remove("ZHIHU_ACCESS_SECRET");
        cmd.arg("ppt").arg("status").arg("ppt_1");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("\"code\":20001"))
            .stderr(predicate::str::contains("Missing access secret"));
    });
}

#[tokio::test]
#[serial]
async fn cli_ppt_generate_against_mock_server_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/ppt-generation/tasks/ppt_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Code": 0, "Message": "success",
            "Data": {
                "task_id": "ppt_1", "task_status": "succeeded", "progress": 1,
                "result": { "url": "https://example.com/deck.pptx", "expires_at_ms": 1782800000000_i64 }
            }
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/ppt-generation/tasks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Code": 0, "Message": "success",
            "Data": { "task_id": "ppt_1", "task_status": "pending" }
        })))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("zhihu").unwrap();
    cmd.env("ZHIHU_ACCESS_SECRET", "fake");
    cmd.env("ZHIHU_OPENAPI_BASE_URL", server.uri());
    cmd.arg("ppt")
        .arg("generate")
        .arg("https://zhuanlan.zhihu.com/p/987654321")
        .arg("--pages")
        .arg("12")
        .arg("--timeout-secs")
        .arg("30");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"task_status\": \"succeeded\""))
        .stdout(predicate::str::contains("deck.pptx"));
}
