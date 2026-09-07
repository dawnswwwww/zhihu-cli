use crate::client::ZhihuClient;
use crate::error::{Result, ZhihuError};
use serde_json::Value;

/// Whether a task-status response envelope is finished.
///
/// Terminal means `Data.task_status` is `succeeded` or `failed`. An envelope
/// with a non-zero `Code` is also treated as terminal so the one-shot
/// commands print it instead of polling a doomed task until timeout.
pub(crate) fn is_terminal(task: &Value) -> bool {
    if task.get("Code").and_then(|c| c.as_i64()) != Some(0) {
        return true;
    }
    matches!(
        task.pointer("/Data/task_status").and_then(|s| s.as_str()),
        Some("succeeded") | Some("failed")
    )
}

/// Poll the task status endpoint (`status_path`) until the task is terminal
/// or `timeout_secs` elapses, sleeping `interval` between polls. Returns the
/// final task JSON on success, `ZhihuError::TaskTimeout` otherwise.
pub(crate) async fn wait_for_task(
    client: &ZhihuClient,
    status_path: &str,
    task_id: &str,
    timeout_secs: u64,
    interval: std::time::Duration,
) -> Result<Value> {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    loop {
        let task = client.get(status_path, &[]).await?;
        if is_terminal(&task) {
            return Ok(task);
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(ZhihuError::TaskTimeout {
                task_id: task_id.to_string(),
                waited_secs: timeout_secs,
            });
        }
        tokio::time::sleep(interval).await;
    }
}

#[cfg(test)]
mod tests {
    //! Tests for the shared async-task polling helper used by the pdf and
    //! ppt one-shot commands.

    use serde_json::json;

    #[test]
    fn is_terminal_true_for_succeeded_and_failed() {
        for status in ["succeeded", "failed"] {
            let task = json!({ "Code": 0, "Data": { "task_status": status } });
            assert!(super::is_terminal(&task), "{status} should be terminal");
        }
    }

    #[test]
    fn is_terminal_false_for_pending_and_running() {
        for status in ["pending", "running"] {
            let task = json!({ "Code": 0, "Data": { "task_status": status } });
            assert!(!super::is_terminal(&task), "{status} should not be terminal");
        }
    }

    #[test]
    fn is_terminal_true_for_nonzero_code_envelope() {
        let task = json!({ "Code": 10001, "Message": "file_id is invalid", "Data": null });
        assert!(super::is_terminal(&task));
    }

    #[test]
    fn is_terminal_false_for_missing_status() {
        let task = json!({ "Code": 0, "Data": null });
        assert!(!super::is_terminal(&task));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn wait_for_task_returns_final_task_when_it_becomes_terminal() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // Mount order matters: later-mounted mocks are matched first, so the
        // "running" mock answers exactly one poll, then "succeeded" takes over.
        Mock::given(method("GET"))
            .and(path("/api/v1/pdf-parse/tasks/t-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0, "Data": { "task_id": "t-1", "task_status": "succeeded", "progress": 1 }
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/v1/pdf-parse/tasks/t-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0, "Data": { "task_id": "t-1", "task_status": "running", "progress": 0.35 }
            })))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let final_task = super::wait_for_task(
            &client,
            "/api/v1/pdf-parse/tasks/t-1",
            "t-1",
            30,
            std::time::Duration::from_millis(1),
        )
        .await
        .unwrap();
        assert_eq!(final_task["Data"]["task_status"], "succeeded");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn wait_for_task_times_out_with_task_timeout_error() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/pdf-parse/tasks/t-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0, "Data": { "task_id": "t-1", "task_status": "running", "progress": 0.1 }
            })))
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let err = super::wait_for_task(
            &client,
            "/api/v1/pdf-parse/tasks/t-1",
            "t-1",
            0,
            std::time::Duration::from_millis(1),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, crate::error::ZhihuError::TaskTimeout { task_id, waited_secs } if task_id == "t-1" && waited_secs == 0));
    }
}
