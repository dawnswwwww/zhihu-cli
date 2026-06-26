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
