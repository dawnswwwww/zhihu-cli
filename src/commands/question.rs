use crate::cli::QuestionCommand;
use crate::client::ZhihuClient;
use crate::error::{Result, ZhihuError};
use crate::output::{dispatch_result, print_error};

pub async fn run(cmd: QuestionCommand) {
    if let Err(e) = dispatch_result(handle(cmd).await) {
        print_error(&e);
    }
}

async fn handle(cmd: QuestionCommand) -> Result<serde_json::Value> {
    handle_with_client(cmd, &ZhihuClient::new()?).await
}

/// Testable inner core: takes the client as a parameter so unit tests can
/// pass a mock. Public within the crate for tests.
pub(crate) async fn handle_with_client(cmd: QuestionCommand, client: &ZhihuClient) -> Result<serde_json::Value> {
    let req = build_request(&cmd)?;
    let query_refs: Vec<(&str, &str)> = req.query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    client.get(req.path, &query_refs).await
}

/// The fully-prepared request for a question command: HTTP path plus the
/// already-validated query parameters. Owned (not borrowed) so the result
/// can outlive the borrowed `QuestionCommand`.
#[derive(Debug, PartialEq)]
pub(crate) struct QuestionRequest {
    pub path: &'static str,
    pub query: Vec<(&'static str, String)>,
}

/// Validate inputs and assemble the (path, query) for a question command.
///
/// `Count` is clamped to `[1, 20]` and `Limit` to `[1, 50]` to match the
/// documented API limits. A blank `--query` is rejected; omitting `--query`
/// entirely yields profile-based recommendations. `Offset` is passed through
/// unvalidated (the server rejects negatives), consistent with `user.rs`.
pub(crate) fn build_request(cmd: &QuestionCommand) -> Result<QuestionRequest> {
    match cmd {
        QuestionCommand::Recommend { query, count } => {
            let mut params = Vec::new();
            if let Some(query) = query {
                let trimmed = query.trim();
                if trimmed.is_empty() {
                    return Err(ZhihuError::InvalidArgument(
                        "--query cannot be empty".to_string(),
                    ));
                }
                params.push(("Query", trimmed.to_string()));
            }
            params.push(("Count", (*count).clamp(1, 20).to_string()));
            Ok(QuestionRequest {
                path: "/api/v1/user/question_recommendations",
                query: params,
            })
        }
        QuestionCommand::Answers {
            question_url,
            offset,
            limit,
        } => {
            let url = question_url.trim();
            if url.is_empty() {
                return Err(ZhihuError::InvalidArgument(
                    "question URL cannot be empty".to_string(),
                ));
            }
            Ok(QuestionRequest {
                path: "/api/v1/content/question_answers",
                query: vec![
                    ("QuestionUrl", url.to_string()),
                    ("Offset", offset.to_string()),
                    ("Limit", (*limit).clamp(1, 50).to_string()),
                ],
            })
        }
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the question command group's pure parameter-assembly
    //! layer and the client-injected handler. The full binary path is
    //! exercised by `tests/cli.rs`.

    use crate::cli::QuestionCommand;
    use crate::error::ZhihuError;

    fn recommend(query: Option<&str>, count: i32) -> QuestionCommand {
        QuestionCommand::Recommend {
            query: query.map(str::to_string),
            count,
        }
    }

    fn answers(url: &str, offset: i64, limit: i64) -> QuestionCommand {
        QuestionCommand::Answers {
            question_url: url.to_string(),
            offset,
            limit,
        }
    }

    #[test]
    fn recommend_request_uses_recommendations_path() {
        let req = super::build_request(&recommend(None, 5)).unwrap();
        assert_eq!(req.path, "/api/v1/user/question_recommendations");
    }

    #[test]
    fn recommend_request_omits_query_param_when_absent() {
        let req = super::build_request(&recommend(None, 5)).unwrap();
        assert_eq!(req.query, vec![("Count", "5".to_string())]);
    }

    #[test]
    fn recommend_request_includes_trimmed_query() {
        let req = super::build_request(&recommend(Some("  rust  "), 7)).unwrap();
        assert_eq!(
            req.query,
            vec![("Query", "rust".to_string()), ("Count", "7".to_string())]
        );
    }

    #[test]
    fn recommend_request_rejects_blank_query() {
        let err = super::build_request(&recommend(Some("   "), 5)).unwrap_err();
        assert!(matches!(err, ZhihuError::InvalidArgument(ref msg) if msg.contains("--query")));
    }

    #[test]
    fn recommend_request_count_clamps_to_one_through_twenty() {
        let assert_count = |count: i32, expected: &str| {
            let req = super::build_request(&recommend(None, count)).unwrap();
            let pair = req.query.iter().find(|(k, _)| *k == "Count").unwrap();
            assert_eq!(pair.1, expected, "count {count} should clamp to {expected}");
        };
        assert_count(0, "1");
        assert_count(-5, "1");
        assert_count(1, "1");
        assert_count(10, "10");
        assert_count(20, "20");
        assert_count(21, "20");
        assert_count(1000, "20");
    }

    #[test]
    fn answers_request_uses_question_answers_path() {
        let req = super::build_request(&answers("https://www.zhihu.com/question/1", 0, 20)).unwrap();
        assert_eq!(req.path, "/api/v1/content/question_answers");
    }

    #[test]
    fn answers_request_assembles_full_query() {
        let req = super::build_request(&answers("https://www.zhihu.com/question/19550225", 40, 30)).unwrap();
        assert_eq!(
            req.query,
            vec![
                ("QuestionUrl", "https://www.zhihu.com/question/19550225".to_string()),
                ("Offset", "40".to_string()),
                ("Limit", "30".to_string()),
            ]
        );
    }

    #[test]
    fn answers_request_rejects_blank_url() {
        let err = super::build_request(&answers("   ", 0, 20)).unwrap_err();
        assert!(matches!(err, ZhihuError::InvalidArgument(ref msg) if msg.contains("question URL")));
    }

    #[test]
    fn answers_request_limit_clamps_to_one_through_fifty() {
        let assert_limit = |limit: i64, expected: &str| {
            let req = super::build_request(&answers("https://www.zhihu.com/question/1", 0, limit)).unwrap();
            let pair = req.query.iter().find(|(k, _)| *k == "Limit").unwrap();
            assert_eq!(pair.1, expected, "limit {limit} should clamp to {expected}");
        };
        assert_limit(0, "1");
        assert_limit(-3, "1");
        assert_limit(1, "1");
        assert_limit(25, "25");
        assert_limit(50, "50");
        assert_limit(51, "50");
        assert_limit(1000, "50");
    }

    #[test]
    fn answers_request_passes_offset_through_unvalidated() {
        // Matches `user.rs`: the server rejects negative offsets.
        let req = super::build_request(&answers("https://www.zhihu.com/question/1", -5, 20)).unwrap();
        let pair = req.query.iter().find(|(k, _)| *k == "Offset").unwrap();
        assert_eq!(pair.1, "-5");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_recommend_calls_endpoint() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/user/question_recommendations"))
            .and(query_param("Query", "rust"))
            .and(query_param("Count", "7"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": { "Items": [] }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let result = super::handle_with_client(recommend(Some("rust"), 7), &client).await.unwrap();
        assert_eq!(result["Code"], 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_recommend_omits_query_when_absent() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param, query_param_is_missing};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/user/question_recommendations"))
            .and(query_param_is_missing("Query"))
            .and(query_param("Count", "5"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": { "Items": [] }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let result = super::handle_with_client(recommend(None, 5), &client).await.unwrap();
        assert_eq!(result["Code"], 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_answers_calls_endpoint() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/content/question_answers"))
            .and(query_param("QuestionUrl", "https://www.zhihu.com/question/19550225"))
            .and(query_param("Offset", "0"))
            .and(query_param("Limit", "20"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": { "Items": [], "Paging": { "IsEnd": true, "Totals": 0 } }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let result = super::handle_with_client(answers("https://www.zhihu.com/question/19550225", 0, 20), &client)
            .await
            .unwrap();
        assert_eq!(result["Code"], 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_propagates_validation_error_without_http() {
        // An unreachable base URL proves no request is attempted: validation
        // must fail before any HTTP call.
        let client = crate::client::ZhihuClient::with_secret_and_base_url(
            "fake".into(),
            "http://127.0.0.1:1".into(),
        );
        let err = super::handle_with_client(recommend(Some("  "), 5), &client)
            .await
            .unwrap_err();
        assert!(matches!(err, ZhihuError::InvalidArgument(_)));
    }
}
