use crate::cli::AskArgs;
use crate::client::ZhihuClient;
use crate::error::{Result, ZhihuError};
use crate::output::{print_error, print_json, print_json_line};
use serde_json::{json, Value};

pub async fn run(args: AskArgs) {
    let stream = args.stream;
    match handle(args).await {
        Ok(value) => {
            if !stream {
                print_json(&value);
            }
        }
        Err(e) => print_error(&e),
    }
}

async fn handle(args: AskArgs) -> Result<Value> {
    let client = ZhihuClient::new()?;
    let body = json!({
        "model": args.model.api_name(),
        "messages": [{"role":"user","content":args.query}],
        "stream": args.stream,
    });

    if args.stream {
        stream_ask(client, body).await
    } else {
        client.post("/v1/chat/completions", body).await
    }
}

async fn stream_ask(client: ZhihuClient, body: Value) -> Result<Value> {
    use futures_util::StreamExt;

    let resp = client
        .request(reqwest::Method::POST, "/v1/chat/completions")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body_text = resp.text().await.unwrap_or_default();
        return Err(ZhihuError::Api {
            status,
            body: body_text,
        });
    }

    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(':') {
                continue;
            }
            if line == "data: [DONE]" {
                continue;
            }
            if let Some(json_str) = line.strip_prefix("data: ") {
                match serde_json::from_str::<Value>(json_str) {
                    Ok(event) => {
                        let delta = event
                            .pointer("/choices/0/delta")
                            .cloned()
                            .unwrap_or_else(|| json!({}));
                        let finish_reason = event
                            .pointer("/choices/0/finish_reason")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        if !delta.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                            print_json_line(&json!({"delta": delta}),
                            );
                        }
                        if let Some(reason) = finish_reason {
                            print_json_line(
                                &json!({"finish_reason": reason}),
                            );
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
    }
    Ok(json!({"status":"stream_complete"}))
}
