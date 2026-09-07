use crate::cli::PdfCommand;
use crate::client::ZhihuClient;
use crate::commands::task_poll::wait_for_task;
use crate::error::{Result, ZhihuError};
use crate::output::{dispatch_result, print_error};
use reqwest::Method;
use serde_json::Value;
use std::time::Duration;

/// Sleep between status polls for the one-shot `pdf parse` command.
const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub async fn run(cmd: PdfCommand) {
    if let Err(e) = dispatch_result(handle(cmd).await) {
        print_error(&e);
    }
}

async fn handle(cmd: PdfCommand) -> Result<Value> {
    handle_with_client(cmd, &ZhihuClient::new()?).await
}

/// Testable inner core: takes the client as a parameter so unit tests can
/// pass a mock. Public within the crate for tests.
pub(crate) async fn handle_with_client(cmd: PdfCommand, client: &ZhihuClient) -> Result<Value> {
    match cmd {
        PdfCommand::Upload { file } => {
            let form = build_upload_form(&file)?;
            client.post_multipart("/resources/v1/files", form).await
        }
        PdfCommand::Task { file_id, idempotency_key } => {
            create_task_with_client(client, &file_id, idempotency_key.as_deref()).await
        }
        PdfCommand::Status { task_id } => {
            client.get(&build_status_path(&task_id), &[]).await
        }
        PdfCommand::Parse { file, timeout_secs, idempotency_key } => {
            parse_with_client(client, &file, timeout_secs, idempotency_key.as_deref(), POLL_INTERVAL).await
        }
    }
}

/// One-shot `pdf parse`: upload the file, create the parse task, then poll
/// until the task is terminal or the timeout elapses. `interval` is a
/// parameter so tests can poll quickly.
pub(crate) async fn parse_with_client(
    client: &ZhihuClient,
    file: &str,
    timeout_secs: u64,
    idempotency_key: Option<&str>,
    interval: Duration,
) -> Result<Value> {
    let form = build_upload_form(file)?;
    let upload = client.post_multipart("/resources/v1/files", form).await?;
    let file_id = upload
        .pointer("/Data/file_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ZhihuError::InvalidArgument("upload response did not contain Data.file_id".to_string())
        })?;

    let task = create_task_with_client(client, file_id, idempotency_key).await?;
    let task_id = task
        .pointer("/Data/task_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ZhihuError::InvalidArgument("task response did not contain Data.task_id".to_string())
        })?;

    wait_for_task(client, &build_status_path(task_id), task_id, timeout_secs, interval).await
}

/// Create a parse task for `file_id`, attaching the optional
/// `Idempotency-Key` header when provided.
async fn create_task_with_client(
    client: &ZhihuClient,
    file_id: &str,
    idempotency_key: Option<&str>,
) -> Result<Value> {
    let mut builder = client
        .request(Method::POST, "/api/v1/pdf-parse/tasks")
        .json(&build_task_body(file_id));
    if let Some(key) = idempotency_key {
        builder = builder.header("Idempotency-Key", key);
    }
    client.send_json(builder).await
}

/// Read the PDF at `file_path` and assemble the multipart form for
/// `pdf upload`. The API expects the file under the `file` form field.
pub(crate) fn build_upload_form(file_path: &str) -> Result<reqwest::multipart::Form> {
    let bytes = std::fs::read(file_path)?;
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_path.to_string());
    let mime = mime_guess::from_path(&file_name).first_or_octet_stream();
    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name(file_name)
        .mime_str(mime.essence_str())?;
    Ok(reqwest::multipart::Form::new().part("file", part))
}

pub(crate) fn build_task_body(file_id: &str) -> Value {
    serde_json::json!({ "file_id": file_id })
}

pub(crate) fn build_status_path(task_id: &str) -> String {
    format!("/api/v1/pdf-parse/tasks/{task_id}")
}

#[cfg(test)]
mod tests {
    //! Unit tests for the pdf command group's pure parameter-assembly layer
    //! and the client-injected handler. The full binary path is exercised
    //! by `tests/cli.rs`.

    use crate::cli::PdfCommand;

    #[test]
    fn task_body_wraps_file_id() {
        let body = super::build_task_body("file_abc");
        assert_eq!(body, serde_json::json!({ "file_id": "file_abc" }));
    }

    #[test]
    fn status_path_embeds_task_id() {
        assert_eq!(
            super::build_status_path("pdf_123"),
            "/api/v1/pdf-parse/tasks/pdf_123"
        );
    }

    #[test]
    fn upload_form_missing_file_maps_to_io_error() {
        let err = super::build_upload_form("/nonexistent/doc.pdf").unwrap_err();
        assert!(matches!(err, crate::error::ZhihuError::Io(_)));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_upload_posts_multipart_file_field() {
        use serde_json::json;
        use wiremock::matchers::{method, path, body_string_contains};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let tmp = tempfile::TempDir::new().unwrap();
        let file_path = tmp.path().join("report.pdf");
        std::fs::write(&file_path, "%PDF-1.4 fake").unwrap();

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/resources/v1/files"))
            .and(body_string_contains("filename=\"report.pdf\""))
            .and(body_string_contains("%PDF-1.4 fake"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0, "Message": "success", "Data": { "file_id": "file_abc" }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let cmd = PdfCommand::Upload { file: file_path.to_string_lossy().to_string() };
        let result = super::handle_with_client(cmd, &client).await.unwrap();
        assert_eq!(result["Data"]["file_id"], "file_abc");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_task_posts_body_and_idempotency_key() {
        use serde_json::json;
        use wiremock::matchers::{method, path, body_partial_json, header};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/pdf-parse/tasks"))
            .and(body_partial_json(json!({ "file_id": "file_abc" })))
            .and(header("Idempotency-Key", "key-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0, "Message": "success",
                "Data": { "task_id": "pdf_1", "task_status": "pending" }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let cmd = PdfCommand::Task { file_id: "file_abc".into(), idempotency_key: Some("key-1".into()) };
        let result = super::handle_with_client(cmd, &client).await.unwrap();
        assert_eq!(result["Data"]["task_id"], "pdf_1");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_status_gets_task_path() {
        use serde_json::json;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/pdf-parse/tasks/pdf_1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0, "Message": "success",
                "Data": { "task_id": "pdf_1", "task_status": "running", "progress": 0.4 }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let cmd = PdfCommand::Status { task_id: "pdf_1".into() };
        let result = super::handle_with_client(cmd, &client).await.unwrap();
        assert_eq!(result["Data"]["task_status"], "running");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn parse_with_client_runs_upload_task_and_wait() {
        use serde_json::json;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let tmp = tempfile::TempDir::new().unwrap();
        let file_path = tmp.path().join("report.pdf");
        std::fs::write(&file_path, "%PDF-1.4 fake").unwrap();

        let server = MockServer::start().await;
        // status: succeeded (mounted first so the one-shot pending mock below wins the first poll)
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
        Mock::given(method("GET"))
            .and(path("/api/v1/pdf-parse/tasks/pdf_1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0, "Message": "success",
                "Data": { "task_id": "pdf_1", "task_status": "pending", "progress": 0 }
            })))
            .up_to_n_times(1)
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

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let final_task = super::parse_with_client(
            &client,
            &file_path.to_string_lossy(),
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
    async fn parse_with_client_rejects_upload_response_without_file_id() {
        use serde_json::json;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let tmp = tempfile::TempDir::new().unwrap();
        let file_path = tmp.path().join("report.pdf");
        std::fs::write(&file_path, "%PDF-1.4 fake").unwrap();

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/resources/v1/files"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0, "Message": "success", "Data": {}
            })))
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let err = super::parse_with_client(
            &client,
            &file_path.to_string_lossy(),
            30,
            None,
            std::time::Duration::from_millis(1),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, crate::error::ZhihuError::InvalidArgument(ref msg) if msg.contains("file_id")));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn parse_with_client_rejects_task_response_without_task_id() {
        use serde_json::json;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let tmp = tempfile::TempDir::new().unwrap();
        let file_path = tmp.path().join("report.pdf");
        std::fs::write(&file_path, "%PDF-1.4 fake").unwrap();

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/pdf-parse/tasks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0, "Message": "success", "Data": {}
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

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let err = super::parse_with_client(
            &client,
            &file_path.to_string_lossy(),
            30,
            None,
            std::time::Duration::from_millis(1),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, crate::error::ZhihuError::InvalidArgument(ref msg) if msg.contains("task_id")));
    }
}
