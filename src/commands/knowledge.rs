use crate::cli::{KbCommand, KbScope, RecallScope};
use crate::client::ZhihuClient;
use crate::error::{Result, ZhihuError};
use crate::output::{dispatch_result, print_error};

pub async fn run(cmd: KbCommand) {
    if let Err(e) = dispatch_result(handle(cmd).await) {
        print_error(&e);
    }
}

async fn handle(cmd: KbCommand) -> Result<serde_json::Value> {
    handle_with_client(cmd, &ZhihuClient::new()?).await
}

/// Testable inner core: takes the client as a parameter so unit tests can
/// pass a mock. Public within the crate for tests.
pub(crate) async fn handle_with_client(cmd: KbCommand, client: &ZhihuClient) -> Result<serde_json::Value> {
    match cmd {
        KbCommand::List { scope } => {
            let req = build_list_request(&scope);
            client.get(req.path, &[( "Scope", req.scope.as_str())]).await
        }
        KbCommand::Items { kb_id, cursor, limit } => {
            let req = build_items_request(&kb_id, cursor.as_deref(), limit);
            let query_refs: Vec<(&str, &str)> =
                req.query.iter().map(|(k, v)| (*k, v.as_str())).collect();
            client.get(&req.path, &query_refs).await
        }
        KbCommand::Search { query, kb_ids, scopes, limit } => {
            let body = build_search_body(&query, &kb_ids, &scopes, limit)?;
            client.post("/api/v1/knowledge/search", body).await
        }
        KbCommand::Upload { file, kb_id } => {
            let form = build_upload_form(&file, kb_id.as_deref())?;
            client.post_multipart("/api/v1/knowledge/files", form).await
        }
    }
}

/// The fully-prepared request for `kb list`.
#[derive(Debug, PartialEq)]
pub(crate) struct KbListRequest {
    pub path: &'static str,
    pub scope: String,
}

pub(crate) fn build_list_request(scope: &KbScope) -> KbListRequest {
    KbListRequest {
        path: "/api/v1/knowledge/bases",
        scope: scope.api_name().to_string(),
    }
}

/// The fully-prepared request for `kb items`: templated path plus the
/// already-validated query parameters.
#[derive(Debug, PartialEq)]
pub(crate) struct KbItemsRequest {
    pub path: String,
    pub query: Vec<(&'static str, String)>,
}

/// `Limit` is clamped to `[1, 20]` to match the Zhihu OpenAPI limit. A blank
/// `Cursor` is omitted so a fresh first page is requested.
pub(crate) fn build_items_request(kb_id: &str, cursor: Option<&str>, limit: i32) -> KbItemsRequest {
    let mut query = Vec::new();
    if let Some(cursor) = cursor.filter(|c| !c.trim().is_empty()) {
        query.push(("Cursor", cursor.to_string()));
    }
    query.push(("Limit", limit.clamp(1, 20).to_string()));
    KbItemsRequest {
        path: format!("/api/v1/knowledge/bases/{kb_id}/items"),
        query,
    }
}

/// Assemble the JSON body for `kb search`.
///
/// The API requires at least one knowledge base ID or recall scope; the
/// query must be non-blank after trimming. `Limit` is clamped to `[1, 10]`.
pub(crate) fn build_search_body(
    query: &str,
    kb_ids: &[String],
    scopes: &[RecallScope],
    limit: i32,
) -> Result<serde_json::Value> {
    let query = query.trim();
    if query.is_empty() {
        return Err(ZhihuError::InvalidArgument(
            "query cannot be empty".to_string(),
        ));
    }
    if kb_ids.is_empty() && scopes.is_empty() {
        return Err(ZhihuError::InvalidArgument(
            "at least one --kb-id or --scope is required".to_string(),
        ));
    }
    Ok(serde_json::json!({
        "Query": query,
        "KnowledgeBaseIDs": kb_ids,
        "RecallScopes": scopes.iter().map(|s| s.api_name()).collect::<Vec<_>>(),
        "Limit": limit.clamp(1, 10),
    }))
}

/// Read the file at `file_path` and assemble the multipart form for
/// `kb upload`. The part filename comes from the path's file name; the MIME
/// type is guessed from the extension.
pub(crate) fn build_upload_form(
    file_path: &str,
    kb_id: Option<&str>,
) -> Result<reqwest::multipart::Form> {
    let bytes = std::fs::read(file_path)?;
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_path.to_string());
    let mime = mime_guess::from_path(&file_name).first_or_octet_stream();
    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name(file_name)
        .mime_str(mime.essence_str())?;
    let mut form = reqwest::multipart::Form::new().part("File", part);
    if let Some(kb_id) = kb_id {
        form = form.text("KnowledgeBaseID", kb_id.to_string());
    }
    Ok(form)
}

#[cfg(test)]
mod tests {
    //! Unit tests for the knowledge base command group's pure
    //! parameter-assembly layer and the client-injected handler. The full
    //! binary path is exercised by `tests/cli.rs`.

    use crate::cli::{KbCommand, KbScope, RecallScope};

    #[test]
    fn kb_scope_api_names() {
        assert_eq!(KbScope::All.api_name(), "all");
        assert_eq!(KbScope::Created.api_name(), "created");
        assert_eq!(KbScope::Subscribed.api_name(), "subscribed");
    }

    #[test]
    fn recall_scope_api_names() {
        assert_eq!(RecallScope::Personal.api_name(), "personal");
        assert_eq!(RecallScope::Subscription.api_name(), "subscription");
        assert_eq!(RecallScope::Public.api_name(), "public");
    }

    #[test]
    fn list_request_uses_scope() {
        let req = super::build_list_request(&KbScope::Subscribed);
        assert_eq!(req.path, "/api/v1/knowledge/bases");
        assert_eq!(req.scope, "subscribed");
    }

    #[test]
    fn items_request_builds_path_and_limit() {
        let req = super::build_items_request("7526139256098382426", None, 20);
        assert_eq!(req.path, "/api/v1/knowledge/bases/7526139256098382426/items");
        assert_eq!(req.query, vec![("Limit", "20".to_string())]);
    }

    #[test]
    fn items_request_includes_cursor_when_provided() {
        let req = super::build_items_request("kb-1", Some("next-cursor"), 20);
        assert_eq!(
            req.query,
            vec![
                ("Cursor", "next-cursor".to_string()),
                ("Limit", "20".to_string()),
            ]
        );
    }

    #[test]
    fn items_request_omits_blank_cursor() {
        let req = super::build_items_request("kb-1", Some("   "), 20);
        assert_eq!(req.query, vec![("Limit", "20".to_string())]);
    }

    #[test]
    fn items_request_limit_clamps_to_one_through_twenty() {
        let assert_limit = |limit: i32, expected: &str| {
            let req = super::build_items_request("kb-1", None, limit);
            let pair = req.query.iter().find(|(k, _)| *k == "Limit").unwrap();
            assert_eq!(pair.1, expected, "limit {limit} should clamp to {expected}");
        };
        assert_limit(0, "1");
        assert_limit(-1, "1");
        assert_limit(21, "20");
        assert_limit(100, "20");
        assert_limit(20, "20");
        assert_limit(15, "15");
    }

    #[test]
    fn search_body_requires_kb_ids_or_scopes() {
        let err = super::build_search_body("q", &[], &[], 10).unwrap_err();
        assert!(matches!(err, crate::error::ZhihuError::InvalidArgument(ref msg) if msg.contains("kb-id")));
    }

    #[test]
    fn search_body_rejects_blank_query() {
        let err = super::build_search_body("   ", &["kb-1".to_string()], &[], 10).unwrap_err();
        assert!(matches!(err, crate::error::ZhihuError::InvalidArgument(ref msg) if msg.contains("query")));
    }

    #[test]
    fn search_body_assembles_full_payload() {
        let body = super::build_search_body(
            " 退款规则 ",
            &["kb-1".to_string(), "kb-2".to_string()],
            &[RecallScope::Personal, RecallScope::Public],
            10,
        )
        .unwrap();
        assert_eq!(body["Query"], "退款规则");
        assert_eq!(body["KnowledgeBaseIDs"], serde_json::json!(["kb-1", "kb-2"]));
        assert_eq!(body["RecallScopes"], serde_json::json!(["personal", "public"]));
        assert_eq!(body["Limit"], 10);
    }

    #[test]
    fn search_body_limit_clamps_to_one_through_ten() {
        let kb = vec!["kb-1".to_string()];
        let assert_limit = |limit: i32, expected: i32| {
            let body = super::build_search_body("q", &kb, &[], limit).unwrap();
            assert_eq!(body["Limit"], expected);
        };
        assert_limit(0, 1);
        assert_limit(-2, 1);
        assert_limit(11, 10);
        assert_limit(99, 10);
        assert_limit(5, 5);
    }

    #[test]
    fn upload_form_missing_file_maps_to_io_error() {
        let err = super::build_upload_form("/nonexistent/file.pdf", None).unwrap_err();
        assert!(matches!(err, crate::error::ZhihuError::Io(_)));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_list_calls_endpoint() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/knowledge/bases"))
            .and(query_param("Scope", "created"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": { "Items": [] }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let cmd = KbCommand::List { scope: KbScope::Created };
        let result = super::handle_with_client(cmd, &client).await.unwrap();
        assert_eq!(result["Code"], 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_items_calls_endpoint() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/knowledge/bases/7526139256098382426/items"))
            .and(query_param("Limit", "20"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": { "Items": [], "Total": 0, "HasMore": false }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let cmd = KbCommand::Items {
            kb_id: "7526139256098382426".into(),
            cursor: None,
            limit: 20,
        };
        let result = super::handle_with_client(cmd, &client).await.unwrap();
        assert_eq!(result["Code"], 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_search_posts_json_body() {
        use serde_json::json;
        use wiremock::matchers::{method, path, body_partial_json};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/knowledge/search"))
            .and(body_partial_json(json!({
                "Query": "退款规则",
                "KnowledgeBaseIDs": ["kb-1"],
                "RecallScopes": [],
                "Limit": 5
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": { "Items": [] }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let cmd = KbCommand::Search {
            query: "退款规则".into(),
            kb_ids: vec!["kb-1".into()],
            scopes: vec![],
            limit: 5,
        };
        let result = super::handle_with_client(cmd, &client).await.unwrap();
        assert_eq!(result["Code"], 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_upload_posts_multipart() {
        use serde_json::json;
        use wiremock::matchers::{method, path, body_string_contains};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let tmp = tempfile::TempDir::new().unwrap();
        let file_path = tmp.path().join("notes.txt");
        std::fs::write(&file_path, "hello knowledge").unwrap();

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/knowledge/files"))
            .and(body_string_contains("filename=\"notes.txt\""))
            .and(body_string_contains("hello knowledge"))
            .and(body_string_contains("7526139256098382426"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": {
                    "KnowledgeBaseID": "7526139256098382426",
                    "RecallContentID": "abc",
                    "FileName": "notes.txt",
                    "FileSize": 16
                }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let cmd = KbCommand::Upload {
            file: file_path.to_string_lossy().to_string(),
            kb_id: Some("7526139256098382426".into()),
        };
        let result = super::handle_with_client(cmd, &client).await.unwrap();
        assert_eq!(result["Code"], 0);
        assert_eq!(result["Data"]["FileName"], "notes.txt");
    }
}
