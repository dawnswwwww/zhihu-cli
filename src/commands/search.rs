use crate::cli::SearchCommand;
use crate::client::ZhihuClient;
use crate::error::Result;
use crate::output::{print_error, print_json};
use serde::Serialize;

pub async fn run(cmd: SearchCommand) {
    if let Err(e) = dispatch_result(handle(cmd).await) {
        print_error(&e);
    }
}

/// Dispatch a command's `Result` to the appropriate output. See the
/// matching helper in `commands::auth` for the rationale.
pub(crate) fn dispatch_result<T: Serialize>(result: Result<T>) -> Result<()> {
    match result {
        Ok(value) => print_json(&value),
        Err(e) => Err(e),
    }
}

async fn handle(cmd: SearchCommand) -> Result<serde_json::Value> {
    handle_with_client(cmd, &ZhihuClient::new()?).await
}

/// Testable inner core: takes the client as a parameter so unit tests can
/// pass a mock. Public within the crate for tests.
pub(crate) async fn handle_with_client(
    cmd: SearchCommand,
    client: &ZhihuClient,
) -> Result<serde_json::Value> {
    let req = build_request(&cmd);
    let query_refs: Vec<(&str, &str)> = req
        .query
        .iter()
        .map(|(k, v)| (*k, v.as_str()))
        .collect();
    client.get(req.path, &query_refs).await
}

/// The fully-prepared request for a search command: HTTP path plus the
/// already-validated query parameters. Owned (not borrowed) so the result
/// can outlive the borrowed `SearchCommand`.
#[derive(Debug, PartialEq)]
pub(crate) struct SearchRequest {
    pub path: &'static str,
    pub query: Vec<(&'static str, String)>,
}

/// Validate inputs and assemble the (path, query) for a search command.
///
/// - `count` is clamped to `[1, 10]` for `Zhihu` and `[1, 20]` for `Global`.
/// - `db.api_name()` is rendered into the `SearchDB` parameter.
/// - `filter` becomes the `Filter` parameter only when `Some`.
///
/// The bounds differ between commands — they come from the Zhihu OpenAPI
/// limits and are easy to get wrong by copy-paste; the unit tests below
/// pin each branch.
pub(crate) fn build_request(cmd: &SearchCommand) -> SearchRequest {
    match cmd {
        SearchCommand::Zhihu { query, count } => SearchRequest {
            path: "/api/v1/content/zhihu_search",
            query: vec![
                ("Query", query.clone()),
                ("Count", (*count).clamp(1, 10).to_string()),
            ],
        },
        SearchCommand::Global {
            query,
            count,
            filter,
            db,
        } => {
            let mut q = vec![
                ("Query", query.clone()),
                ("Count", (*count).clamp(1, 20).to_string()),
                ("SearchDB", db.api_name().to_string()),
            ];
            if let Some(f) = filter {
                q.push(("Filter", f.clone()));
            }
            SearchRequest {
                path: "/api/v1/content/global_search",
                query: q,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for `build_request` — the pure parameter-assembly layer
    //! of the search commands. The HTTP layer is exercised by `mocked_api.rs`.

    use super::{build_request, dispatch_result};
    use crate::cli::{SearchCommand, SearchDb};
    use crate::client::ZhihuClient;
    use crate::error::{Result, ZhihuError};
    use serde::{Serialize, Serializer};
    use serde_json::Value;

    struct AlwaysFails;
    impl Serialize for AlwaysFails {
        fn serialize<S: Serializer>(
            &self,
            _s: S,
        ) -> std::result::Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("intentional test failure"))
        }
    }

    #[test]
    fn dispatch_result_propagates_serialize_error() {
        let result: Result<&AlwaysFails> = Ok(&AlwaysFails);
        let err = dispatch_result(result).expect_err("AlwaysFails should not serialize");
        assert!(matches!(err, ZhihuError::InvalidArgument(_)));
    }

    #[test]
    fn dispatch_result_returns_err_for_input_err() {
        let result: Result<Value> = Err(ZhihuError::MissingSecret);
        let err = dispatch_result(result).expect_err("Err should propagate");
        assert!(matches!(err, ZhihuError::MissingSecret));
    }

    // ---- handle_with_client (with wiremock) ----

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_zhihu_search_calls_zhihu_search_endpoint() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/content/zhihu_search"))
            .and(query_param("Query", "rust"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0, "Message": "ok", "Data": []
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let cmd = SearchCommand::Zhihu {
            query: "rust".into(),
            count: 5,
        };
        let result = super::handle_with_client(cmd, &client).await.unwrap();
        assert_eq!(result["Code"], 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_global_search_calls_global_search_endpoint() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/content/global_search"))
            .and(query_param("Query", "rust"))
            .and(query_param("SearchDB", "realtime"))
            .and(query_param("Filter", "host==\"x.com\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0, "Message": "ok", "Data": []
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let cmd = SearchCommand::Global {
            query: "rust".into(),
            count: 10,
            filter: Some("host==\"x.com\"".into()),
            db: SearchDb::Realtime,
        };
        let result = super::handle_with_client(cmd, &client).await.unwrap();
        assert_eq!(result["Code"], 0);
    }

    fn pairs(req: &super::SearchRequest) -> Vec<(&str, &str)> {
        req.query.iter().map(|(k, v)| (*k, v.as_str())).collect()
    }

    // 1. Zhihu search hits the zhihu_search endpoint.
    #[test]
    fn zhihu_search_uses_zhihu_search_path() {
        let cmd = SearchCommand::Zhihu {
            query: "rust".into(),
            count: 5,
        };
        let req = build_request(&cmd);
        assert_eq!(req.path, "/api/v1/content/zhihu_search");
    }

    // 2. Zhihu count clamps to the [1, 10] range — both ends.
    #[test]
    fn zhihu_count_clamps_to_one_through_ten() {
        let make = |c: i32| SearchCommand::Zhihu {
            query: "q".into(),
            count: c,
        };
        let assert_count = |c: i32, expected: &str| {
            let req = build_request(&make(c));
            assert_eq!(pairs(&req)[1].1, expected);
        };
        assert_count(0, "1");
        assert_count(-7, "1");
        assert_count(5, "5");
        assert_count(10, "10");
        assert_count(11, "10");
        assert_count(1000, "10");
    }

    // 3. Zhihu's first query param is the user's query string.
    #[test]
    fn zhihu_query_param_is_user_input() {
        let cmd = SearchCommand::Zhihu {
            query: "async rust".into(),
            count: 3,
        };
        let req = build_request(&cmd);
        let p = pairs(&req);
        assert_eq!(p[0], ("Query", "async rust"));
    }

    // 4. Global search hits the global_search endpoint.
    #[test]
    fn global_search_uses_global_search_path() {
        let cmd = SearchCommand::Global {
            query: "q".into(),
            count: 5,
            filter: None,
            db: SearchDb::All,
        };
        let req = build_request(&cmd);
        assert_eq!(req.path, "/api/v1/content/global_search");
    }

    // 5. Global count clamps to [1, 20] — DIFFERENT upper bound than Zhihu.
    #[test]
    fn global_count_clamps_to_one_through_twenty() {
        let make = |c: i32| SearchCommand::Global {
            query: "q".into(),
            count: c,
            filter: None,
            db: SearchDb::All,
        };
        let assert_count = |c: i32, expected: &str| {
            let req = build_request(&make(c));
            assert_eq!(pairs(&req)[1].1, expected);
        };
        assert_count(0, "1");
        assert_count(15, "15");
        assert_count(20, "20");
        assert_count(21, "20");
        assert_count(9999, "20");
    }

    // 6. SearchDB carries the api_name for each variant.
    #[test]
    fn global_search_db_is_renders_api_name() {
        let cases = [
            (SearchDb::All, "all"),
            (SearchDb::Realtime, "realtime"),
            (SearchDb::Static, "static"),
        ];
        for (db, expected) in cases {
            let cmd = SearchCommand::Global {
                query: "q".into(),
                count: 5,
                filter: None,
                db,
            };
            let req = build_request(&cmd);
            let p = pairs(&req);
            let search_db = p.iter().find(|(k, _)| *k == "SearchDB").unwrap();
            assert_eq!(search_db.1, expected, "db variant {:?} wrong", db);
        }
    }

    // 7. Filter param appears only when Some.
    #[test]
    fn global_filter_param_omitted_when_none() {
        let cmd = SearchCommand::Global {
            query: "q".into(),
            count: 5,
            filter: None,
            db: SearchDb::All,
        };
        let req = build_request(&cmd);
        let p = pairs(&req);
        assert!(
            p.iter().all(|(k, _)| *k != "Filter"),
            "Filter must be absent when input is None, got: {p:?}"
        );
    }

    #[test]
    fn global_filter_param_present_when_some() {
        let cmd = SearchCommand::Global {
            query: "q".into(),
            count: 5,
            filter: Some("host==\"example.com\"".into()),
            db: SearchDb::All,
        };
        let req = build_request(&cmd);
        let p = pairs(&req);
        let filter = p
            .iter()
            .find(|(k, _)| *k == "Filter")
            .expect("Filter param should be present");
        assert_eq!(filter.1, "host==\"example.com\"");
    }
}
