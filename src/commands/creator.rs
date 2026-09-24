use crate::cli::CreatorCommand;
use crate::client::ZhihuClient;
use crate::error::{Result, ZhihuError};
use crate::output::{dispatch_result, print_error};

pub async fn run(cmd: CreatorCommand) {
    if let Err(e) = dispatch_result(handle(cmd).await) {
        print_error(&e);
    }
}

async fn handle(cmd: CreatorCommand) -> Result<serde_json::Value> {
    handle_with_client(cmd, &ZhihuClient::new()?).await
}

/// Testable inner core: takes the client as a parameter so unit tests can
/// pass a mock. Public within the crate for tests.
pub(crate) async fn handle_with_client(cmd: CreatorCommand, client: &ZhihuClient) -> Result<serde_json::Value> {
    let req = build_request(&cmd)?;
    let query_refs: Vec<(&str, &str)> = req.query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    client.get(req.path, &query_refs).await
}

/// The fully-prepared request for a creator command: HTTP path plus the
/// already-validated query parameters. Owned (not borrowed) so the result
/// can outlive the borrowed `CreatorCommand`.
#[derive(Debug, PartialEq)]
pub(crate) struct CreatorRequest {
    pub path: &'static str,
    pub query: Vec<(&'static str, String)>,
}

/// Validate inputs and assemble the (path, query) for a creator command.
///
/// `Limit` is clamped to `[1, 50]` to match the documented API limit.
/// `Offset` is passed through unvalidated (the server rejects negatives),
/// consistent with `user.rs`. `StartDate`/`EndDate` follow the paired-date
/// rules enforced by [`date_range_params`].
pub(crate) fn build_request(cmd: &CreatorCommand) -> Result<CreatorRequest> {
    match cmd {
        CreatorCommand::Detail { content_url } => Ok(CreatorRequest {
            path: "/api/v1/user/content_detail",
            query: vec![("ContentUrl", require_non_blank(content_url, "content URL")?)],
        }),
        CreatorCommand::Comments {
            content_url,
            offset,
            limit,
            order,
        } => Ok(CreatorRequest {
            path: "/api/v1/user/content_comments",
            query: vec![
                ("ContentUrl", require_non_blank(content_url, "content URL")?),
                ("Offset", offset.to_string()),
                ("Limit", (*limit).clamp(1, 50).to_string()),
                ("Order", order.api_name().to_string()),
            ],
        }),
        CreatorCommand::AccountStats {
            content_type,
            start_date,
            end_date,
        } => {
            let mut query = vec![("ContentType", content_type.api_name().to_string())];
            query.extend(date_range_params(start_date.as_deref(), end_date.as_deref())?);
            Ok(CreatorRequest {
                path: "/api/v1/user/creator_account_stats",
                query,
            })
        }
        CreatorCommand::ContentStats {
            content_url,
            start_date,
            end_date,
        } => {
            let mut query = vec![("ContentUrl", require_non_blank(content_url, "content URL")?)];
            query.extend(date_range_params(start_date.as_deref(), end_date.as_deref())?);
            Ok(CreatorRequest {
                path: "/api/v1/user/creator_content_stats",
                query,
            })
        }
    }
}

/// Trim a required free-form argument; blank-after-trim is an error.
fn require_non_blank(value: &str, label: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ZhihuError::InvalidArgument(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(trimmed.to_string())
}

/// Assemble the optional `StartDate`/`EndDate` pair. The dates must be given
/// together, be strict `YYYY-MM-DD`, and the end must not precede the start.
fn date_range_params(start_date: Option<&str>, end_date: Option<&str>) -> Result<Vec<(&'static str, String)>> {
    match (start_date, end_date) {
        (None, None) => Ok(Vec::new()),
        (Some(_), None) | (None, Some(_)) => Err(ZhihuError::InvalidArgument(
            "--start-date and --end-date must be provided together".to_string(),
        )),
        (Some(start), Some(end)) => {
            if !is_valid_date(start) {
                return Err(ZhihuError::InvalidArgument(format!(
                    "invalid --start-date {start:?}: expected YYYY-MM-DD"
                )));
            }
            if !is_valid_date(end) {
                return Err(ZhihuError::InvalidArgument(format!(
                    "invalid --end-date {end:?}: expected YYYY-MM-DD"
                )));
            }
            if end < start {
                return Err(ZhihuError::InvalidArgument(
                    "--end-date must not be earlier than --start-date".to_string(),
                ));
            }
            Ok(vec![
                ("StartDate", start.to_string()),
                ("EndDate", end.to_string()),
            ])
        }
    }
}

/// Strict `YYYY-MM-DD` shape with a plausible month (01-12) and day (01-31).
/// Hand-rolled to avoid a date-parsing dependency.
fn is_valid_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let digits = |slice: &[u8]| slice.iter().all(u8::is_ascii_digit);
    if !digits(&b[0..4]) || !digits(&b[5..7]) || !digits(&b[8..10]) {
        return false;
    }
    let month = u32::from(b[5] - b'0') * 10 + u32::from(b[6] - b'0');
    let day = u32::from(b[8] - b'0') * 10 + u32::from(b[9] - b'0');
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

#[cfg(test)]
mod tests {
    //! Unit tests for the creator command group's pure parameter-assembly
    //! layer and the client-injected handler. The full binary path is
    //! exercised by `tests/cli.rs`.

    use crate::cli::{CommentOrder, CreatorCommand, CreatorContentType};
    use crate::error::ZhihuError;

    fn detail(url: &str) -> CreatorCommand {
        CreatorCommand::Detail {
            content_url: url.to_string(),
        }
    }

    fn comments(url: &str, offset: i64, limit: i64, order: CommentOrder) -> CreatorCommand {
        CreatorCommand::Comments {
            content_url: url.to_string(),
            offset,
            limit,
            order,
        }
    }

    fn account_stats(content_type: CreatorContentType, start: Option<&str>, end: Option<&str>) -> CreatorCommand {
        CreatorCommand::AccountStats {
            content_type,
            start_date: start.map(str::to_string),
            end_date: end.map(str::to_string),
        }
    }

    fn content_stats(url: &str, start: Option<&str>, end: Option<&str>) -> CreatorCommand {
        CreatorCommand::ContentStats {
            content_url: url.to_string(),
            start_date: start.map(str::to_string),
            end_date: end.map(str::to_string),
        }
    }

    // ---- detail ----

    #[test]
    fn detail_request_uses_content_detail_path() {
        let req = super::build_request(&detail("https://www.zhihu.com/answer/1")).unwrap();
        assert_eq!(req.path, "/api/v1/user/content_detail");
        assert_eq!(
            req.query,
            vec![("ContentUrl", "https://www.zhihu.com/answer/1".to_string())]
        );
    }

    #[test]
    fn detail_request_trims_content_url() {
        let req = super::build_request(&detail("  https://www.zhihu.com/answer/1  ")).unwrap();
        assert_eq!(
            req.query,
            vec![("ContentUrl", "https://www.zhihu.com/answer/1".to_string())]
        );
    }

    #[test]
    fn detail_request_rejects_blank_url() {
        let err = super::build_request(&detail("   ")).unwrap_err();
        assert!(matches!(err, ZhihuError::InvalidArgument(ref msg) if msg.contains("content URL")));
    }

    // ---- comments ----

    #[test]
    fn comments_request_uses_content_comments_path() {
        let req = super::build_request(&comments("https://www.zhihu.com/answer/1", 0, 20, CommentOrder::Score)).unwrap();
        assert_eq!(req.path, "/api/v1/user/content_comments");
    }

    #[test]
    fn comments_request_assembles_full_query() {
        let req = super::build_request(&comments("https://www.zhihu.com/answer/1", 10, 30, CommentOrder::Reverse)).unwrap();
        assert_eq!(
            req.query,
            vec![
                ("ContentUrl", "https://www.zhihu.com/answer/1".to_string()),
                ("Offset", "10".to_string()),
                ("Limit", "30".to_string()),
                ("Order", "reverse".to_string()),
            ]
        );
    }

    #[test]
    fn comments_request_rejects_blank_url() {
        let err = super::build_request(&comments("  ", 0, 20, CommentOrder::Score)).unwrap_err();
        assert!(matches!(err, ZhihuError::InvalidArgument(ref msg) if msg.contains("content URL")));
    }

    #[test]
    fn comments_request_limit_clamps_to_one_through_fifty() {
        let assert_limit = |limit: i64, expected: &str| {
            let req = super::build_request(&comments("https://www.zhihu.com/answer/1", 0, limit, CommentOrder::Score)).unwrap();
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
    fn comments_request_passes_offset_through_unvalidated() {
        // Matches `user.rs`: the server rejects negative offsets.
        let req = super::build_request(&comments("https://www.zhihu.com/answer/1", -5, 20, CommentOrder::Score)).unwrap();
        let pair = req.query.iter().find(|(k, _)| *k == "Offset").unwrap();
        assert_eq!(pair.1, "-5");
    }

    // ---- account-stats ----

    #[test]
    fn account_stats_request_uses_account_stats_path() {
        let req = super::build_request(&account_stats(CreatorContentType::All, None, None)).unwrap();
        assert_eq!(req.path, "/api/v1/user/creator_account_stats");
    }

    #[test]
    fn account_stats_request_omits_dates_when_absent() {
        let req = super::build_request(&account_stats(CreatorContentType::Article, None, None)).unwrap();
        assert_eq!(req.query, vec![("ContentType", "article".to_string())]);
    }

    #[test]
    fn account_stats_request_includes_date_range_when_given() {
        let req = super::build_request(&account_stats(
            CreatorContentType::Answer,
            Some("2026-01-01"),
            Some("2026-01-31"),
        ))
        .unwrap();
        assert_eq!(
            req.query,
            vec![
                ("ContentType", "answer".to_string()),
                ("StartDate", "2026-01-01".to_string()),
                ("EndDate", "2026-01-31".to_string()),
            ]
        );
    }

    // ---- content-stats ----

    #[test]
    fn content_stats_request_uses_content_stats_path() {
        let req = super::build_request(&content_stats("https://zhuanlan.zhihu.com/p/1", None, None)).unwrap();
        assert_eq!(req.path, "/api/v1/user/creator_content_stats");
    }

    #[test]
    fn content_stats_request_assembles_query_with_dates() {
        let req = super::build_request(&content_stats(
            "https://zhuanlan.zhihu.com/p/1",
            Some("2026-02-01"),
            Some("2026-02-28"),
        ))
        .unwrap();
        assert_eq!(
            req.query,
            vec![
                ("ContentUrl", "https://zhuanlan.zhihu.com/p/1".to_string()),
                ("StartDate", "2026-02-01".to_string()),
                ("EndDate", "2026-02-28".to_string()),
            ]
        );
    }

    #[test]
    fn content_stats_request_rejects_blank_url() {
        let err = super::build_request(&content_stats("  ", None, None)).unwrap_err();
        assert!(matches!(err, ZhihuError::InvalidArgument(ref msg) if msg.contains("content URL")));
    }

    // ---- date-range validation (shared by account-stats / content-stats) ----

    #[test]
    fn date_range_rejects_solo_start_date() {
        let err = super::build_request(&account_stats(CreatorContentType::All, Some("2026-01-01"), None)).unwrap_err();
        assert!(matches!(err, ZhihuError::InvalidArgument(ref msg) if msg.contains("provided together")));
    }

    #[test]
    fn date_range_rejects_solo_end_date() {
        let err = super::build_request(&content_stats("https://zhuanlan.zhihu.com/p/1", None, Some("2026-01-31"))).unwrap_err();
        assert!(matches!(err, ZhihuError::InvalidArgument(ref msg) if msg.contains("provided together")));
    }

    #[test]
    fn date_range_rejects_end_earlier_than_start() {
        let err = super::build_request(&account_stats(
            CreatorContentType::All,
            Some("2026-01-31"),
            Some("2026-01-01"),
        ))
        .unwrap_err();
        assert!(matches!(err, ZhihuError::InvalidArgument(ref msg) if msg.contains("earlier")));
    }

    #[test]
    fn date_range_accepts_equal_start_and_end() {
        let req = super::build_request(&account_stats(
            CreatorContentType::All,
            Some("2026-01-15"),
            Some("2026-01-15"),
        ))
        .unwrap();
        assert_eq!(
            req.query,
            vec![
                ("ContentType", "all".to_string()),
                ("StartDate", "2026-01-15".to_string()),
                ("EndDate", "2026-01-15".to_string()),
            ]
        );
    }

    #[test]
    fn date_range_rejects_malformed_start_date() {
        for bad in [
            "2026-1-1",    // not zero-padded
            "2026/01/01",  // wrong separator
            "20260101",    // no separators
            "2026-13-01",  // month out of range
            "2026-00-01",  // month zero
            "2026-01-32",  // day out of range
            "2026-01-00",  // day zero
            "abcd-ef-gh",  // non-digits
            " 2026-01-01", // leading whitespace
        ] {
            let err = super::build_request(&account_stats(CreatorContentType::All, Some(bad), Some("2026-12-31")))
                .unwrap_err();
            assert!(
                matches!(err, ZhihuError::InvalidArgument(ref msg) if msg.contains("--start-date")),
                "start date {bad:?} should be rejected"
            );
        }
    }

    #[test]
    fn date_range_rejects_malformed_end_date() {
        // Valid start + malformed end must name --end-date.
        for bad in ["2026-1-31", "2026/01/31", "2026-13-31", "2026-01-32"] {
            let err = super::build_request(&account_stats(CreatorContentType::All, Some("2026-01-01"), Some(bad)))
                .unwrap_err();
            assert!(
                matches!(err, ZhihuError::InvalidArgument(ref msg) if msg.contains("--end-date")),
                "end date {bad:?} should be rejected"
            );
        }
    }

    // ---- wiremock handler tests ----

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_detail_calls_endpoint() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/user/content_detail"))
            .and(query_param("ContentUrl", "https://www.zhihu.com/answer/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": { "Content": "" }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let result = super::handle_with_client(detail("https://www.zhihu.com/answer/1"), &client).await.unwrap();
        assert_eq!(result["Code"], 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_comments_calls_endpoint() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/user/content_comments"))
            .and(query_param("ContentUrl", "https://www.zhihu.com/answer/1"))
            .and(query_param("Offset", "10"))
            .and(query_param("Limit", "30"))
            .and(query_param("Order", "ascending"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": { "Items": [], "Paging": { "IsEnd": true, "Totals": 0 } }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let result = super::handle_with_client(
            comments("https://www.zhihu.com/answer/1", 10, 30, CommentOrder::Ascending),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(result["Code"], 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_account_stats_calls_endpoint() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/user/creator_account_stats"))
            .and(query_param("ContentType", "answer"))
            .and(query_param("StartDate", "2026-01-01"))
            .and(query_param("EndDate", "2026-01-31"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": {}
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let result = super::handle_with_client(
            account_stats(CreatorContentType::Answer, Some("2026-01-01"), Some("2026-01-31")),
            &client,
        )
        .await
        .unwrap();
        assert_eq!(result["Code"], 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_account_stats_omits_dates_when_absent() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param, query_param_is_missing};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/user/creator_account_stats"))
            .and(query_param("ContentType", "all"))
            .and(query_param_is_missing("StartDate"))
            .and(query_param_is_missing("EndDate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": {}
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let result = super::handle_with_client(account_stats(CreatorContentType::All, None, None), &client)
            .await
            .unwrap();
        assert_eq!(result["Code"], 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_content_stats_calls_endpoint() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/user/creator_content_stats"))
            .and(query_param("ContentUrl", "https://zhuanlan.zhihu.com/p/1"))
            .and(query_param("StartDate", "2026-02-01"))
            .and(query_param("EndDate", "2026-02-28"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": {}
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let result = super::handle_with_client(
            content_stats("https://zhuanlan.zhihu.com/p/1", Some("2026-02-01"), Some("2026-02-28")),
            &client,
        )
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
        let err = super::handle_with_client(
            account_stats(CreatorContentType::All, Some("2026-01-01"), None),
            &client,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, ZhihuError::InvalidArgument(_)));
    }
}
