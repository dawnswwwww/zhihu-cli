use serial_test::serial;
use std::env;

fn get_secret() -> Option<String> {
    env::var("ZHIHU_ACCESS_SECRET").ok().filter(|s| !s.trim().is_empty())
}

#[tokio::test]
#[serial]
async fn search_zhihu_returns_results() {
    let Some(secret) = get_secret() else { return };
    let client = zhihu_cli::client::ZhihuClient::with_secret_and_base_url(
        secret,
        "https://developer.zhihu.com".into(),
    );
    let resp = client
        .get("/api/v1/content/zhihu_search", &[("Query", "RAG"), ("Count", "3")])
        .await
        .expect("search zhihu should succeed");
    assert_eq!(resp.get("Code"), Some(&serde_json::json!(0)));
    assert!(resp.get("Data").is_some());
}

#[tokio::test]
#[serial]
async fn search_global_returns_results() {
    let Some(secret) = get_secret() else { return };
    let client = zhihu_cli::client::ZhihuClient::with_secret_and_base_url(
        secret,
        "https://developer.zhihu.com".into(),
    );
    let resp = client
        .get(
            "/api/v1/content/global_search",
            &[("Query", "人工智能"), ("Count", "3"), ("SearchDB", "all")],
        )
        .await
        .expect("search global should succeed");
    assert_eq!(resp.get("Code"), Some(&serde_json::json!(0)));
    assert!(resp.get("Data").is_some());
}

#[tokio::test]
#[serial]
async fn ask_non_stream_returns_completion() {
    let Some(secret) = get_secret() else { return };
    let client = zhihu_cli::client::ZhihuClient::with_secret_and_base_url(
        secret,
        "https://developer.zhihu.com".into(),
    );
    let body = serde_json::json!({
        "model": "zhida-fast-1p5",
        "messages": [{"role": "user", "content": "你好"}],
        "stream": false,
    });
    let resp = client
        .post("/v1/chat/completions", body)
        .await
        .expect("ask should succeed");
    assert!(resp.get("choices").is_some());
}

#[tokio::test]
#[serial]
async fn ask_stream_returns_chunks() {
    let Some(secret) = get_secret() else { return };
    let client = zhihu_cli::client::ZhihuClient::with_secret_and_base_url(
        secret,
        "https://developer.zhihu.com".into(),
    );

    let req = client
        .request(reqwest::Method::POST, "/v1/chat/completions")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "zhida-fast-1p5",
            "messages": [{"role": "user", "content": "你好"}],
            "stream": true,
        }));

    let resp = req.send().await.expect("stream request should send");
    assert!(resp.status().is_success());

    use futures_util::StreamExt;
    let mut stream = resp.bytes_stream();
    let mut saw_data = false;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.expect("chunk should be ok");
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            if line.starts_with("data: ") && line != "data: [DONE]" {
                saw_data = true;
                let json_str = line.strip_prefix("data: ").unwrap();
                let _: serde_json::Value =
                    serde_json::from_str(json_str).expect("chunk should be valid JSON");
            }
        }
    }
    assert!(saw_data, "should have received at least one data chunk");
}

#[tokio::test]
#[serial]
async fn hot_list_returns_results() {
    let Some(secret) = get_secret() else { return };
    let client = zhihu_cli::client::ZhihuClient::with_secret_and_base_url(
        secret,
        "https://developer.zhihu.com".into(),
    );
    let resp = client
        .get("/api/v1/content/hot_list", &[("Limit", "3")])
        .await
        .expect("hot list should succeed");
    assert_eq!(resp.get("Code"), Some(&serde_json::json!(0)));
    let data = resp.get("Data").expect("Data should exist");
    let total = data.get("Total").and_then(|v| v.as_i64()).unwrap_or(0);
    let items_len = data
        .get("Items")
        .and_then(|v| v.as_array())
        .map(|arr| arr.len())
        .unwrap_or(0);
    assert!(items_len > 0, "hot list should return at least one item");
    assert!(
        items_len <= total as usize,
        "returned items should not exceed reported total"
    );
}
