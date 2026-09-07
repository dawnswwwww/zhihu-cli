use crate::cli::QuotaArgs;
use crate::client::ZhihuClient;
use crate::error::Result;
use crate::output::{dispatch_result, print_error};

pub async fn run(args: QuotaArgs) {
    if let Err(e) = dispatch_result(handle(args).await) {
        print_error(&e);
    }
}

async fn handle(args: QuotaArgs) -> Result<serde_json::Value> {
    handle_with_client(args, &ZhihuClient::new()?).await
}

/// Testable inner core: takes the client as a parameter so unit tests can
/// pass a mock. Public within the crate for tests.
pub(crate) async fn handle_with_client(args: QuotaArgs, client: &ZhihuClient) -> Result<serde_json::Value> {
    let req = build_request(&args);
    let query_refs: Vec<(&str, &str)> = req.query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    client.get(req.path, &query_refs).await
}

/// The fully-prepared request for the quota command: HTTP path plus the
/// already-validated query parameters. Owned (not borrowed) so the result
/// can outlive the borrowed `QuotaArgs`.
#[derive(Debug, PartialEq)]
pub(crate) struct QuotaRequest {
    pub path: &'static str,
    pub query: Vec<(&'static str, String)>,
}

/// Validate inputs and assemble the (path, query) for the quota command.
///
/// Empty `APIIDs` is omitted so the API returns every quota item; blank
/// segments in `--ids` are dropped.
pub(crate) fn build_request(args: &QuotaArgs) -> QuotaRequest {
    let ids: Vec<String> = args
        .ids
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let query = if ids.is_empty() {
        Vec::new()
    } else {
        vec![("APIIDs", ids.join(","))]
    };
    QuotaRequest {
        path: "/api/v1/quota",
        query,
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the quota command's pure parameter-assembly layer and
    //! the client-injected handler. The full binary path is exercised by
    //! `tests/cli.rs`.

    use crate::cli::QuotaArgs;

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_calls_quota_endpoint() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/quota"))
            .and(query_param("APIIDs", "knowledge,zhihu_search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": [{ "APIID": "knowledge", "RemainingQuota": 488 }]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let args = QuotaArgs { ids: vec!["knowledge".into(), "zhihu_search".into()] };
        let result = super::handle_with_client(args, &client).await.unwrap();
        assert_eq!(result["Code"], 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_omits_apiids_when_no_ids() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param_is_missing};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/quota"))
            .and(query_param_is_missing("APIIDs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": []
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let args = QuotaArgs { ids: vec![] };
        let result = super::handle_with_client(args, &client).await.unwrap();
        assert_eq!(result["Code"], 0);
    }

    #[test]
    fn quota_request_uses_quota_path() {
        let args = QuotaArgs { ids: vec![] };
        let req = super::build_request(&args);
        assert_eq!(req.path, "/api/v1/quota");
    }

    #[test]
    fn quota_request_omits_apiids_for_empty_ids() {
        let args = QuotaArgs { ids: vec![] };
        let req = super::build_request(&args);
        assert!(req.query.is_empty());
    }

    #[test]
    fn quota_request_joins_ids_with_comma() {
        let args = QuotaArgs { ids: vec!["knowledge".into(), "zhihu_search".into()] };
        let req = super::build_request(&args);
        assert_eq!(req.query, vec![("APIIDs", "knowledge,zhihu_search".to_string())]);
    }

    #[test]
    fn quota_request_trims_and_drops_empty_ids() {
        let args = QuotaArgs { ids: vec!["  knowledge ".into(), "".into(), "tools".into()] };
        let req = super::build_request(&args);
        assert_eq!(req.query, vec![("APIIDs", "knowledge,tools".to_string())]);
    }
}
