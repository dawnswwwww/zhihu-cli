use crate::cli::PptCommand;
use crate::client::ZhihuClient;
use crate::commands::task_poll::wait_for_task;
use crate::error::{Result, ZhihuError};
use crate::output::{dispatch_result, print_error};
use reqwest::Method;
use serde_json::Value;
use std::time::Duration;

/// Sleep between status polls for the one-shot `ppt generate` command.
const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub async fn run(cmd: PptCommand) {
    if let Err(e) = dispatch_result(handle(cmd).await) {
        print_error(&e);
    }
}

async fn handle(cmd: PptCommand) -> Result<Value> {
    handle_with_client(cmd, &ZhihuClient::new()?).await
}

/// Testable inner core: takes the client as a parameter so unit tests can
/// pass a mock. Public within the crate for tests.
pub(crate) async fn handle_with_client(cmd: PptCommand, client: &ZhihuClient) -> Result<Value> {
    match cmd {
        PptCommand::Task { resource_url, pages, idempotency_key } => {
            create_task_with_client(client, &resource_url, pages, idempotency_key.as_deref()).await
        }
        PptCommand::Status { task_id } => {
            client.get(&build_status_path(&task_id), &[]).await
        }
        PptCommand::Generate { resource_url, pages, timeout_secs, idempotency_key } => {
            generate_with_client(
                client,
                &resource_url,
                pages,
                timeout_secs,
                idempotency_key.as_deref(),
                POLL_INTERVAL,
            )
            .await
        }
    }
}

/// One-shot `ppt generate`: create the generation task, then poll until the
/// task is terminal or the timeout elapses. `interval` is a parameter so
/// tests can poll quickly.
pub(crate) async fn generate_with_client(
    client: &ZhihuClient,
    resource_url: &str,
    pages: i32,
    timeout_secs: u64,
    idempotency_key: Option<&str>,
    interval: Duration,
) -> Result<Value> {
    let task = create_task_with_client(client, resource_url, pages, idempotency_key).await?;
    let task_id = task
        .pointer("/Data/task_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ZhihuError::InvalidArgument("task response did not contain Data.task_id".to_string())
        })?;

    wait_for_task(client, &build_status_path(task_id), task_id, timeout_secs, interval).await
}

/// Create a generation task for `resource_url`, attaching the optional
/// `Idempotency-Key` header when provided.
async fn create_task_with_client(
    client: &ZhihuClient,
    resource_url: &str,
    pages: i32,
    idempotency_key: Option<&str>,
) -> Result<Value> {
    let mut builder = client
        .request(Method::POST, "/api/v1/ppt-generation/tasks")
        .json(&build_task_body(resource_url, pages));
    if let Some(key) = idempotency_key {
        builder = builder.header("Idempotency-Key", key);
    }
    client.send_json(builder).await
}

/// Assemble the JSON body for `ppt task`. `num_pages` is clamped to
/// `[6, 21]` to match the Zhihu OpenAPI range.
pub(crate) fn build_task_body(resource_url: &str, pages: i32) -> Value {
    serde_json::json!({
        "resource_url": resource_url,
        "num_pages": pages.clamp(6, 21),
    })
}

pub(crate) fn build_status_path(task_id: &str) -> String {
    format!("/api/v1/ppt-generation/tasks/{task_id}")
}

#[cfg(test)]
mod tests {
    //! Unit tests for the ppt command group's pure parameter-assembly layer
    //! and the client-injected handler. The full binary path is exercised
    //! by `tests/cli.rs`.

    use crate::cli::PptCommand;

    const ANSWER_URL: &str = "https://www.zhihu.com/question/1892249263213356127/answer/2021688002292752412";

    #[test]
    fn task_body_wraps_url_and_pages() {
        let body = super::build_task_body(ANSWER_URL, 12);
        assert_eq!(
            body,
            serde_json::json!({ "resource_url": ANSWER_URL, "num_pages": 12 })
        );
    }

    #[test]
    fn task_body_pages_clamp_to_six_through_twenty_one() {
        let assert_pages = |pages: i32, expected: i32| {
            let body = super::build_task_body(ANSWER_URL, pages);
            assert_eq!(body["num_pages"], expected, "pages {pages} should clamp to {expected}");
        };
        assert_pages(1, 6);
        assert_pages(5, 6);
        assert_pages(6, 6);
        assert_pages(12, 12);
        assert_pages(21, 21);
        assert_pages(22, 21);
        assert_pages(100, 21);
    }

    #[test]
    fn status_path_embeds_task_id() {
        assert_eq!(
            super::build_status_path("ppt_123"),
            "/api/v1/ppt-generation/tasks/ppt_123"
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_task_posts_body_and_idempotency_key() {
        use serde_json::json;
        use wiremock::matchers::{method, path, body_partial_json, header};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/ppt-generation/tasks"))
            .and(body_partial_json(json!({ "resource_url": ANSWER_URL, "num_pages": 12 })))
            .and(header("Idempotency-Key", "key-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0, "Message": "success",
                "Data": { "task_id": "ppt_1", "task_status": "pending" }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let cmd = PptCommand::Task {
            resource_url: ANSWER_URL.into(),
            pages: 12,
            idempotency_key: Some("key-1".into()),
        };
        let result = super::handle_with_client(cmd, &client).await.unwrap();
        assert_eq!(result["Data"]["task_id"], "ppt_1");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_status_gets_task_path() {
        use serde_json::json;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/ppt-generation/tasks/ppt_1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0, "Message": "success",
                "Data": { "task_id": "ppt_1", "task_status": "running", "progress": 0.45 }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let cmd = PptCommand::Status { task_id: "ppt_1".into() };
        let result = super::handle_with_client(cmd, &client).await.unwrap();
        assert_eq!(result["Data"]["task_status"], "running");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn generate_with_client_creates_task_and_waits() {
        use serde_json::json;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // status: succeeded (mounted first so the one-shot pending mock below wins the first poll)
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
        Mock::given(method("GET"))
            .and(path("/api/v1/ppt-generation/tasks/ppt_1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0, "Message": "success",
                "Data": { "task_id": "ppt_1", "task_status": "pending", "progress": 0 }
            })))
            .up_to_n_times(1)
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

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let final_task = super::generate_with_client(
            &client,
            ANSWER_URL,
            12,
            30,
            None,
            std::time::Duration::from_millis(1),
        )
        .await
        .unwrap();
        assert_eq!(final_task["Data"]["task_status"], "succeeded");
        assert!(final_task["Data"]["result"]["url"].is_string());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn generate_with_client_rejects_task_response_without_task_id() {
        use serde_json::json;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/ppt-generation/tasks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0, "Message": "success", "Data": {}
            })))
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let err = super::generate_with_client(
            &client,
            ANSWER_URL,
            12,
            30,
            None,
            std::time::Duration::from_millis(1),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, crate::error::ZhihuError::InvalidArgument(ref msg) if msg.contains("task_id")));
    }
}
