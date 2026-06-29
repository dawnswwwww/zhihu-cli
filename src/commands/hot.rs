use crate::cli::HotArgs;
use crate::client::ZhihuClient;
use crate::error::Result;
use crate::output::{print_error, print_json};
use serde::Serialize;

pub async fn run(args: HotArgs) {
    if let Err(e) = dispatch_result(handle(args).await) {
        print_error(&e);
    }
}

/// Dispatch a command's `Result` to the appropriate output. See the
/// matching helper in `commands::search` for the rationale.
pub(crate) fn dispatch_result<T: Serialize>(result: Result<T>) -> Result<()> {
    match result {
        Ok(value) => print_json(&value),
        Err(e) => Err(e),
    }
}

async fn handle(args: HotArgs) -> Result<serde_json::Value> {
    handle_with_client(args, &ZhihuClient::new()?).await
}

/// Testable inner core: takes the client as a parameter so unit tests can
/// pass a mock. Public within the crate for tests.
pub(crate) async fn handle_with_client(args: HotArgs, client: &ZhihuClient) -> Result<serde_json::Value> {
    let req = build_request(&args);
    let query_refs: Vec<(&str, &str)> = req.query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    client.get(req.path, &query_refs).await
}

/// The fully-prepared request for the hot list command: HTTP path plus the
/// already-validated query parameters. Owned (not borrowed) so the result
/// can outlive the borrowed `HotArgs`.
#[derive(Debug, PartialEq)]
pub(crate) struct HotRequest {
    pub path: &'static str,
    pub query: Vec<(&'static str, String)>,
}

/// Validate inputs and assemble the (path, query) for the hot list command.
///
/// `limit` is clamped to `[1, 30]` to match the Zhihu OpenAPI limit.
pub(crate) fn build_request(args: &HotArgs) -> HotRequest {
    HotRequest {
        path: "/api/v1/content/hot_list",
        query: vec![("Limit", args.limit.clamp(1, 30).to_string())],
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for `build_request` — the pure parameter-assembly layer
    //! of the hot list command. The HTTP layer is exercised by `mocked_api.rs`.

    use super::build_request;
    use crate::cli::HotArgs;
    use crate::error::{Result, ZhihuError};
    use serde::{Serialize, Serializer};

    struct AlwaysFails;
    impl Serialize for AlwaysFails {
        fn serialize<S: Serializer>(&self, _s: S) -> std::result::Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("intentional test failure"))
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_calls_hot_list_endpoint() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/content/hot_list"))
            .and(query_param("Limit", "10"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "ok",
                "Data": { "Total": 1, "Items": [] }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let args = HotArgs { limit: 10 };
        let result = super::handle_with_client(args, &client).await.unwrap();
        assert_eq!(result["Code"], 0);
    }

    #[test]
    fn dispatch_result_propagates_serialize_error() {
        let result: Result<&AlwaysFails> = Ok(&AlwaysFails);
        let err = super::dispatch_result(result).expect_err("AlwaysFails should not serialize");
        assert!(matches!(err, ZhihuError::InvalidArgument(_)));
    }

    #[test]
    fn dispatch_result_returns_err_for_input_err() {
        let result: Result<serde_json::Value> = Err(ZhihuError::MissingSecret);
        let err = super::dispatch_result(result).expect_err("Err should propagate");
        assert!(matches!(err, ZhihuError::MissingSecret));
    }

    fn pairs(req: &super::HotRequest) -> Vec<(&str, &str)> {
        req.query.iter().map(|(k, v)| (*k, v.as_str())).collect()
    }

    #[test]
    fn hot_request_uses_hot_list_path() {
        let args = HotArgs { limit: 10 };
        let req = build_request(&args);
        assert_eq!(req.path, "/api/v1/content/hot_list");
    }

    #[test]
    fn hot_request_limit_defaults_to_thirty() {
        let args = HotArgs { limit: 30 };
        let req = build_request(&args);
        let limit = pairs(&req).into_iter().find(|(k, _)| *k == "Limit").unwrap();
        assert_eq!(limit.1, "30");
    }

    #[test]
    fn hot_request_limit_clamps_to_one_through_thirty() {
        let make = |limit: i32| HotArgs { limit };
        let assert_limit = |limit: i32, expected: &str| {
            let req = build_request(&make(limit));
            let pair = pairs(&req).into_iter().find(|(k, _)| *k == "Limit").unwrap();
            assert_eq!(pair.1, expected, "limit {limit} should clamp to {expected}");
        };
        assert_limit(0, "1");
        assert_limit(-5, "1");
        assert_limit(1, "1");
        assert_limit(15, "15");
        assert_limit(30, "30");
        assert_limit(31, "30");
        assert_limit(1000, "30");
    }
}
