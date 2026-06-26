use crate::config::Config;
use crate::error::{Result, ZhihuError};
use reqwest::{Client, Method, RequestBuilder};
use serde_json::Value;

pub const DEFAULT_BASE_URL: &str = "https://developer.zhihu.com";

pub struct ZhihuClient {
    client: Client,
    secret: String,
    base_url: String,
}

impl ZhihuClient {
    pub fn new() -> Result<Self> {
        let secret = Config::resolve_secret()?;
        let base_url = std::env::var("ZHIHU_OPENAPI_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
            .trim_end_matches('/')
            .to_string();
        Ok(Self {
            client: Client::new(),
            secret,
            base_url,
        })
    }

    fn auth_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        let auth = format!("Bearer {}", self.secret);
        headers.insert(
            "Authorization",
            auth.parse().expect("Bearer header should be valid"),
        );
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();
        headers.insert(
            "X-Request-Timestamp",
            ts.parse().expect("timestamp should be valid header"),
        );
        headers
    }

    pub fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .request(method, &url)
            .headers(self.auth_headers())
    }

    pub async fn send_json(&self, builder: RequestBuilder) -> Result<Value> {
        let resp = builder.send().await?;
        let status = resp.status();
        let body_text = resp.text().await?;
        if !status.is_success() {
            return Err(ZhihuError::Api {
                status,
                body: body_text,
            });
        }
        serde_json::from_str(&body_text).map_err(|_| ZhihuError::NonJsonResponse)
    }

    pub async fn get(&self, path: &str, query: &[(&str, &str)]) -> Result<Value> {
        let builder = self.request(Method::GET, path).query(query);
        self.send_json(builder).await
    }

    pub async fn post(&self, path: &str, body: Value) -> Result<Value> {
        let builder = self
            .request(Method::POST, path)
            .header("Content-Type", "application/json")
            .json(&body);
        self.send_json(builder).await
    }
}
