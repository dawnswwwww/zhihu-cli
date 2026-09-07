use serial_test::serial;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use zhihu_cli::client::ZhihuClient;
use zhihu_cli::error::ZhihuError;

fn set_base_url(url: &str) {
    unsafe { std::env::set_var("ZHIHU_OPENAPI_BASE_URL", url); }
}

fn clear_base_url() {
    unsafe { std::env::remove_var("ZHIHU_OPENAPI_BASE_URL"); }
}

#[tokio::test]
#[serial]
async fn handles_401_auth_failure() {
    let server = MockServer::start().await;
    set_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/api/v1/content/zhihu_search"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
    let err = client
        .get("/api/v1/content/zhihu_search", &[("Query", "x"), ("Count", "1")])
        .await
        .unwrap_err();

    clear_base_url();
    assert!(matches!(err, ZhihuError::Api { status, .. } if status == 401));
}

#[tokio::test]
#[serial]
async fn handles_403_forbidden() {
    let server = MockServer::start().await;
    set_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/api/v1/content/zhihu_search"))
        .respond_with(ResponseTemplate::new(403).set_body_string("Forbidden"))
        .mount(&server)
        .await;

    let client = ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
    let err = client
        .get("/api/v1/content/zhihu_search", &[("Query", "x"), ("Count", "1")])
        .await
        .unwrap_err();

    clear_base_url();
    assert!(matches!(err, ZhihuError::Api { status, .. } if status == 403));
}

#[tokio::test]
#[serial]
async fn handles_500_server_error() {
    let server = MockServer::start().await;
    set_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/api/v1/content/zhihu_search"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let client = ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
    let err = client
        .get("/api/v1/content/zhihu_search", &[("Query", "x"), ("Count", "1")])
        .await
        .unwrap_err();

    clear_base_url();
    assert!(matches!(err, ZhihuError::Api { status, .. } if status == 500));
}

#[tokio::test]
#[serial]
async fn handles_non_json_response() {
    let server = MockServer::start().await;
    set_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/api/v1/content/zhihu_search"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let client = ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
    let err = client
        .get("/api/v1/content/zhihu_search", &[("Query", "x"), ("Count", "1")])
        .await
        .unwrap_err();

    clear_base_url();
    assert!(matches!(err, ZhihuError::NonJsonResponse));
}

#[tokio::test]
#[serial]
async fn request_includes_auth_headers() {
    let server = MockServer::start().await;
    set_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/api/v1/content/zhihu_search"))
        .and(wiremock::matchers::header_exists("Authorization"))
        .and(wiremock::matchers::header_exists("X-Request-Timestamp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Code": 0,
            "Message": "success",
            "Data": {}
        })))
        .mount(&server)
        .await;

    let client = ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
    let resp = client
        .get("/api/v1/content/zhihu_search", &[("Query", "x"), ("Count", "1")])
        .await;

    clear_base_url();
    assert!(resp.is_ok());
}

/// Matches requests where the named header is NOT present.
/// wiremock 0.6 ships no `header_absent`, so this fills the gap.
struct HeaderAbsent(&'static str);
impl wiremock::Match for HeaderAbsent {
    fn matches(&self, request: &wiremock::Request) -> bool {
        !request.headers.contains_key(self.0)
    }
}

#[tokio::test]
#[serial]
async fn post_multipart_sends_file_and_fields() {
    use reqwest::multipart;

    let server = MockServer::start().await;
    set_base_url(&server.uri());

    Mock::given(method("POST"))
        .and(path("/api/v1/knowledge/files"))
        .and(wiremock::matchers::body_string_contains(
            "filename=\"notes.txt\"",
        ))
        .and(wiremock::matchers::body_string_contains("KnowledgeBaseID"))
        .and(wiremock::matchers::body_string_contains("hello world"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Code": 0,
            "Message": "success",
            "Data": { "RecallContentID": "abc" }
        })))
        .mount(&server)
        .await;

    let part = multipart::Part::bytes(b"hello world".to_vec())
        .file_name("notes.txt")
        .mime_str("text/plain")
        .unwrap();
    let form = multipart::Form::new()
        .part("File", part)
        .text("KnowledgeBaseID", "7526139256098382426");

    let client = ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
    let resp = client
        .post_multipart("/api/v1/knowledge/files", form)
        .await
        .expect("multipart POST should succeed");

    clear_base_url();
    assert_eq!(resp["Code"], 0);
    assert_eq!(resp["Data"]["RecallContentID"], "abc");
}

#[tokio::test]
#[serial]
async fn post_multipart_maps_error_status_to_api_error() {
    use reqwest::multipart;

    let server = MockServer::start().await;
    set_base_url(&server.uri());

    Mock::given(method("POST"))
        .and(path("/api/v1/knowledge/files"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let form = multipart::Form::new().text("KnowledgeBaseID", "123");
    let client = ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
    let err = client
        .post_multipart("/api/v1/knowledge/files", form)
        .await
        .unwrap_err();

    clear_base_url();
    assert!(matches!(err, ZhihuError::Api { status, .. } if status == 401));
}

#[tokio::test]
#[serial]
async fn get_with_oauth_sends_oauth_header_when_provided() {
    let server = MockServer::start().await;
    set_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/api/v1/user/contents"))
        .and(wiremock::matchers::header("X-OAuth-Token", "oauth-tok"))
        .and(wiremock::matchers::header_exists("Authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Code": 0,
            "Message": "success",
            "Data": { "Items": [], "Paging": { "IsEnd": true } }
        })))
        .mount(&server)
        .await;

    let client = ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
    let resp = client
        .get_with_oauth("/api/v1/user/contents", &[("ContentType", "all")], Some("oauth-tok"))
        .await;

    clear_base_url();
    assert_eq!(resp.unwrap()["Code"], 0);
}

#[tokio::test]
#[serial]
async fn get_with_oauth_omits_oauth_header_when_none() {
    let server = MockServer::start().await;
    set_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/api/v1/user/contents"))
        .and(HeaderAbsent("X-OAuth-Token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Code": 0,
            "Message": "success",
            "Data": { "Items": [], "Paging": { "IsEnd": true } }
        })))
        .mount(&server)
        .await;

    let client = ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
    let resp = client
        .get_with_oauth("/api/v1/user/contents", &[("ContentType", "all")], None)
        .await;

    clear_base_url();
    assert_eq!(resp.unwrap()["Code"], 0);
}

#[tokio::test]
#[serial]
async fn post_sends_json_body_and_returns_response() {
    use serde_json::json;

    let server = MockServer::start().await;
    set_base_url(&server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(wiremock::matchers::header("Content-Type", "application/json"))
        .and(wiremock::matchers::header_exists("Authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "chatcmpl-1",
            "choices": [{"message": {"role": "assistant", "content": "hi"}}],
        })))
        .mount(&server)
        .await;

    let client = ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
    let resp = client
        .post(
            "/v1/chat/completions",
            json!({
                "model": "zhida-fast-1p5",
                "messages": [{"role": "user", "content": "hello"}],
                "stream": false,
            }),
        )
        .await
        .expect("POST should succeed");

    clear_base_url();
    assert_eq!(resp["choices"][0]["message"]["content"], "hi");
}
