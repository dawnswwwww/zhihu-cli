use crate::cli::AskArgs;
use crate::client::ZhihuClient;
use crate::error::{Result, ZhihuError};
use crate::output::{print_error, print_json, print_json_line};
use serde_json::{json, Value};

pub async fn run(args: AskArgs) {
    let stream = args.stream;
    if let Err(e) = print_ask_result(handle(args).await, stream) {
        print_error(&e);
    }
}

/// Decide what to do with `handle`'s result given whether streaming was
/// requested. Pure: returns a `Result` instead of exiting, so all four
/// (stream × ok/err) combinations are unit-testable.
pub(crate) fn print_ask_result(result: Result<Value>, stream: bool) -> Result<()> {
    match (result, stream) {
        // Streaming: results were printed incrementally inside `stream_ask`;
        // the `Ok` value is a sentinel that we drop on the floor.
        (Ok(_), true) => Ok(()),
        (Ok(value), false) => print_json(&value),
        (Err(e), _) => Err(e),
    }
}

async fn handle(args: AskArgs) -> Result<Value> {
    handle_with_client(args, &ZhihuClient::new()?).await
}

/// Testable inner core: takes the client as a parameter so unit tests can
/// pass a mock. Public within the crate for tests.
pub(crate) async fn handle_with_client(
    args: AskArgs,
    client: &ZhihuClient,
) -> Result<Value> {
    let body = build_ask_body(&args);

    if args.stream {
        stream_ask_with_client(client, body).await
    } else {
        client.post("/v1/chat/completions", body).await
    }
}

/// Assemble the JSON request body for a `/v1/chat/completions` call.
///
/// Pure function: takes a borrow of the args, returns an owned `Value`.
/// Extracted from `handle` so the body shape can be unit-tested without
/// touching `ZhihuClient` or the network.
pub(crate) fn build_ask_body(args: &AskArgs) -> Value {
    json!({
        "model": args.model.api_name(),
        "messages": [{"role": "user", "content": &args.query}],
        "stream": args.stream,
    })
}

#[cfg(test)]
mod build_ask_body_tests {
    //! Unit tests for `build_ask_body`. Kept in a nested module to keep
    //! the `sse_parser` tests easy to scan in isolation.

    use super::build_ask_body;
    use crate::cli::{AskArgs, ModelTier};
    use serde_json::json;

    fn args(query: &str, model: ModelTier, stream: bool) -> AskArgs {
        AskArgs {
            query: query.to_string(),
            model,
            stream,
        }
    }

    // 1. Fast model maps to its API name.
    #[test]
    fn fast_model_uses_fast_api_name() {
        let body = build_ask_body(&args("hi", ModelTier::Fast, false));
        assert_eq!(body["model"], "zhida-fast-1p5");
    }

    // 2. Thinking model maps to its API name.
    #[test]
    fn thinking_model_uses_thinking_api_name() {
        let body = build_ask_body(&args("hi", ModelTier::Thinking, false));
        assert_eq!(body["model"], "zhida-thinking-1p5");
    }

    // 3. Agent model maps to its API name.
    #[test]
    fn agent_model_uses_agent_api_name() {
        let body = build_ask_body(&args("hi", ModelTier::Agent, false));
        assert_eq!(body["model"], "zhida-agent");
    }

    // 4. `stream: true` is reflected in the body.
    #[test]
    fn stream_true_is_reflected_in_body() {
        let body = build_ask_body(&args("hi", ModelTier::Fast, true));
        assert_eq!(body["stream"], true);
    }

    // 5. `stream: false` is reflected in the body.
    #[test]
    fn stream_false_is_reflected_in_body() {
        let body = build_ask_body(&args("hi", ModelTier::Fast, false));
        assert_eq!(body["stream"], false);
    }

    // 6. Messages is an array of exactly one user message.
    #[test]
    fn messages_is_single_user_message() {
        let body = build_ask_body(&args("hi", ModelTier::Fast, false));
        let messages = body["messages"]
            .as_array()
            .expect("messages must be an array");
        assert_eq!(messages.len(), 1, "expected exactly one message");
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"], "hi");
    }

    // 7. Multi-word query is preserved verbatim (no whitespace coercion).
    #[test]
    fn multi_word_query_preserved() {
        let body = build_ask_body(&args("hello  world", ModelTier::Fast, false));
        assert_eq!(body["messages"][0]["content"], "hello  world");
    }

    // 8. Unicode query is preserved.
    #[test]
    fn unicode_query_preserved() {
        let body = build_ask_body(&args("你好，世界", ModelTier::Fast, false));
        assert_eq!(body["messages"][0]["content"], "你好，世界");
    }

    // 9. Body is a JSON object with exactly three top-level keys.
    #[test]
    fn body_has_exactly_three_top_level_keys() {
        let body = build_ask_body(&args("hi", ModelTier::Fast, false));
        let obj = body.as_object().expect("body must be an object");
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec!["messages", "model", "stream"],
            "unexpected top-level keys"
        );
        // Sanity: matches the historical shape used by `tests/integration.rs`.
        assert_eq!(
            body,
            json!({
                "model": "zhida-fast-1p5",
                "messages": [{"role": "user", "content": "hi"}],
                "stream": false,
            })
        );
    }
}

/// Streaming version of `handle_with_client`. Takes a `&ZhihuClient` so
/// tests can wiremock the HTTP layer.
pub(crate) async fn stream_ask_with_client(client: &ZhihuClient, body: Value) -> Result<Value> {
    use futures_util::StreamExt;

    let resp = client
        .request(reqwest::Method::POST, "/v1/chat/completions")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body_text = resp.text().await.unwrap_or_default();
        return Err(ZhihuError::Api {
            status,
            body: body_text,
        });
    }

    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            for event in sse_parser::parse_sse_line(line) {
                match event {
                    sse_parser::JsonEvent::Delta(delta) => {
                        // Streaming output: serialization failures mid-stream
                        // are unrecoverable noise; drop them.
                        let _ = print_json_line(&json!({"delta": delta}));
                    }
                    sse_parser::JsonEvent::Finish(reason) => {
                        let _ = print_json_line(&json!({"finish_reason": reason}));
                    }
                }
            }
        }
    }
    Ok(json!({"status":"stream_complete"}))
}

pub mod sse_parser {
    use serde_json::Value;

    /// One semantic event extracted from a single SSE `data:` line.
    #[derive(Debug, Clone, PartialEq)]
    pub enum JsonEvent {
        Delta(Value),
        Finish(String),
    }

    /// Parse a single SSE line into zero, one, or two `JsonEvent`s.
    pub fn parse_sse_line(line: &str) -> Vec<JsonEvent> {
        let line = line.trim();
        if line.is_empty() || line.starts_with(':') {
            return Vec::new();
        }
        if line == "data: [DONE]" {
            return Vec::new();
        }
        let Some(json_str) = line.strip_prefix("data: ") else {
            return Vec::new();
        };
        let Ok(event) = serde_json::from_str::<Value>(json_str) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        let delta = event
            .pointer("/choices/0/delta")
            .cloned()
            .unwrap_or_else(|| Value::Object(Default::default()));
        if !delta.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            out.push(JsonEvent::Delta(delta));
        }
        if let Some(reason) = event
            .pointer("/choices/0/finish_reason")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
        {
            out.push(JsonEvent::Finish(reason));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the SSE frame parser.
    //!
    //! These exercise every branch in `sse_parser::parse_sse_line`: blank
    //! lines, comment lines, the `data: [DONE]` sentinel, delta-only /
    //! finish-only / both events, empty-delta suppression, malformed JSON,
    //! non-`data:` lines, the `data:` (no-space) edge case, and Unicode
    //! content. The matching integration test (`tests/integration.rs::
    //! ask_stream_returns_chunks`) validates behavior against the real
    //! Zhihu streaming endpoint when `ZHIHU_ACCESS_SECRET` is set.

    use serde_json::{json, Value};
    use super::sse_parser::{parse_sse_line, JsonEvent};

    fn delta(content: &str) -> Value {
        json!({ "content": content })
    }

    // 1. Empty input yields no events.
    #[test]
    fn empty_line_yields_no_events() {
        assert_eq!(parse_sse_line(""), Vec::<JsonEvent>::new());
    }

    // 2. Whitespace-only input is treated as empty.
    #[test]
    fn whitespace_only_line_yields_no_events() {
        assert_eq!(parse_sse_line("   "), Vec::<JsonEvent>::new());
    }

    // 3. SSE comment lines (starting with ':') are skipped.
    #[test]
    fn comment_line_is_skipped() {
        assert_eq!(parse_sse_line(":heartbeat"), Vec::<JsonEvent>::new());
    }

    // 4. The `data: [DONE]` sentinel terminates the stream silently.
    #[test]
    fn done_marker_is_skipped() {
        assert_eq!(parse_sse_line("data: [DONE]"), Vec::<JsonEvent>::new());
    }

    // 5. A data line with a non-empty delta yields exactly one Delta event
    //    whose payload is the inner `choices[0].delta` object.
    #[test]
    fn data_with_delta_yields_one_delta_event() {
        let line = r#"data: {"choices":[{"delta":{"content":"hi"}}]}"#;
        assert_eq!(parse_sse_line(line), vec![JsonEvent::Delta(delta("hi"))]);
    }

    // 6. A data line with only a finish_reason yields exactly one Finish event.
    #[test]
    fn data_with_finish_reason_yields_one_finish_event() {
        let line = r#"data: {"choices":[{"finish_reason":"stop"}]}"#;
        assert_eq!(
            parse_sse_line(line),
            vec![JsonEvent::Finish("stop".to_string())]
        );
    }

    // 7. A data line carrying both delta and finish_reason yields BOTH events
    //    in the order: Delta first, then Finish. This codifies the order
    //    produced by the original stream_ask loop.
    #[test]
    fn data_with_delta_and_finish_reason_yields_both_in_order() {
        let line = r#"data: {"choices":[{"delta":{"content":"x"},"finish_reason":"stop"}]}"#;
        assert_eq!(
            parse_sse_line(line),
            vec![
                JsonEvent::Delta(delta("x")),
                JsonEvent::Finish("stop".to_string()),
            ]
        );
    }

    // 8. A data line whose delta is `{}` and which has no finish_reason
    //    produces nothing — matches the `is_empty()` guard in original code.
    #[test]
    fn data_with_empty_delta_and_no_finish_yields_nothing() {
        let line = r#"data: {"choices":[{"delta":{}}]}"#;
        assert_eq!(parse_sse_line(line), Vec::<JsonEvent>::new());
    }

    // 9. Malformed JSON in a `data:` line is silently skipped — original
    //    code's `Err(_) => continue`.
    #[test]
    fn malformed_json_is_skipped() {
        assert_eq!(
            parse_sse_line("data: {not valid json"),
            Vec::<JsonEvent>::new()
        );
    }

    // 10. Lines without a `data: ` (with-space) prefix are ignored.
    #[test]
    fn line_without_data_prefix_is_ignored() {
        assert_eq!(parse_sse_line("event: foo"), Vec::<JsonEvent>::new());
    }

    // 11. The current parser uses `strip_prefix("data: ")` (with space), so
    //     a `data:` line WITHOUT a trailing space is rejected. This locks in
    //     existing behavior — if we later want to accept both forms, this
    //     test will fail and force an explicit decision.
    #[test]
    fn data_prefix_without_space_is_rejected() {
        let line = r#"data:{"choices":[{"delta":{"content":"noSpace"}}]}"#;
        assert_eq!(parse_sse_line(line), Vec::<JsonEvent>::new());
    }

    // 12. Unicode content survives the parse (no lossy conversion before the
    //     JSON decode, since serde_json handles UTF-8 directly).
    #[test]
    fn unicode_content_in_delta_is_preserved() {
        let line = r#"data: {"choices":[{"delta":{"content":"中文"}}]}"#;
        assert_eq!(
            parse_sse_line(line),
            vec![JsonEvent::Delta(delta("中文"))]
        );
    }
}

#[cfg(test)]
mod run_tests {
    //! Tests for `print_ask_result` (the testable core of `run`) and the
    //! `dispatch_result` helper. The actual `run` body is a thin wrapper
    //! around `print_ask_result` + `print_error`, exercised end-to-end by
    //! `tests/cli.rs` and `tests/integration.rs`.

    use super::print_ask_result;
    use crate::cli::{AskArgs, ModelTier};
    use crate::client::ZhihuClient;
    use crate::error::{Result, ZhihuError};
    use serde_json::{json, Value};

    // ---- print_ask_result ----

    #[test]
    fn print_ask_result_streams_ok_drops_sentinel() {
        let result: Result<Value> = Ok(json!({"status": "stream_complete"}));
        assert!(print_ask_result(result, true).is_ok());
    }

    #[test]
    fn print_ask_result_non_stream_ok_prints() {
        let result: Result<Value> = Ok(json!({"id": "x", "choices": []}));
        assert!(print_ask_result(result, false).is_ok());
    }

    #[test]
    fn print_ask_result_non_stream_err_propagates() {
        let result: Result<Value> = Err(ZhihuError::MissingSecret);
        let err = print_ask_result(result, false).expect_err("Err should propagate");
        assert!(matches!(err, ZhihuError::MissingSecret));
    }

    #[test]
    fn print_ask_result_stream_err_propagates() {
        let result: Result<Value> = Err(ZhihuError::Api {
            status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            body: "boom".into(),
        });
        let err = print_ask_result(result, true).expect_err("Err should propagate even in stream mode");
        assert!(matches!(err, ZhihuError::Api { .. }));
    }

    // ---- handle_with_client / stream_ask_with_client (with wiremock) ----

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_non_stream_post() {
        use serde_json::json;
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "x", "choices": [{"message": {"content": "hi"}}],
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let args = AskArgs {
            query: "hello".into(),
            model: ModelTier::Fast,
            stream: false,
        };
        let result = super::handle_with_client(args, &client).await.unwrap();
        assert_eq!(result["choices"][0]["message"]["content"], "hi");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn stream_ask_with_client_parses_sse_stream() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let sse_body = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n",
            "\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\" \"}}]}\n",
            "\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"world\"}}]}\n",
            "\n",
            "data: {\"choices\":[{\"finish_reason\":\"stop\"}]}\n",
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

        let client = ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let args = AskArgs {
            query: "hi".into(),
            model: ModelTier::Fast,
            stream: true,
        };
        let body = json!({
            "model": args.model.api_name(),
            "messages": [{"role": "user", "content": &args.query}],
            "stream": true,
        });
        let result = super::stream_ask_with_client(&client, body).await.unwrap();
        assert_eq!(result["status"], "stream_complete");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn stream_ask_with_client_returns_api_error_on_non_2xx() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(503).set_body_string("overloaded"))
            .expect(1)
            .mount(&server)
            .await;

        let client = ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let body = json!({
            "model": "zhida-fast-1p5",
            "messages": [{"role": "user", "content": "hi"}],
            "stream": true,
        });
        let err = super::stream_ask_with_client(&client, body)
            .await
            .expect_err("non-2xx should fail");
        assert!(
            matches!(err, ZhihuError::Api { status, body } if status == 503 && body == "overloaded")
        );
    }
}
