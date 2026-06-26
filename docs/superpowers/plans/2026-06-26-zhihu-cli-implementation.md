# 知乎 CLI 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现一个 Rust CLI `zhihu`，封装知乎开放平台的搜索与直答接口，支持鉴权配置、JSON 输出与流式响应。

**架构：** 采用分层结构：CLI 解析（`clap`）→ 命令分发 → 配置/凭证解析 → HTTP 客户端（`reqwest`）→ 知乎 API。所有命令输出统一为 JSON，错误以 JSON 形式返回并伴随非零退出码。

**Tech Stack:** Rust, clap, reqwest, tokio, serde, serde_json, toml, dirs, thiserror, futures-util

---

## 文件结构

| 文件 | 职责 |
|------|------|
| `Cargo.toml` | 项目依赖 |
| `src/main.rs` | 程序入口、错误兜底 |
| `src/cli.rs` | `clap` 派生命令定义 |
| `src/error.rs` | 统一错误类型 |
| `src/config.rs` | 配置加载、保存、凭证解析 |
| `src/client.rs` | HTTP 客户端、鉴权头注入 |
| `src/output.rs` | JSON 输出与错误输出辅助函数 |
| `src/commands/mod.rs` | 命令模块声明 |
| `src/commands/auth.rs` | `auth login` / `set-secret` / `status` |
| `src/commands/search.rs` | `search zhihu` / `search global` |
| `src/commands/ask.rs` | `ask` 非流式与流式 |

---

## Task 1: 初始化依赖

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: 添加依赖**

```toml
[package]
name = "zhihu-cli"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = { version = "4", features = ["derive"] }
dirs = "5"
futures-util = "0.3"
reqwest = { version = "0.12", features = ["json", "stream"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
tokio = { version = "1", features = ["full"] }
toml = "0.8"
```

- [ ] **Step 2: 检查编译通过**

Run: `cargo check`
Expected: 无错误（仅有空 main 警告可忽略）

- [ ] **Step 3: 提交**

```bash
git add Cargo.toml
git commit -m "chore: add CLI dependencies

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 2: 统一错误类型

**Files:**
- Create: `src/error.rs`

- [ ] **Step 1: 编写错误类型**

```rust
use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum ZhihuError {
    #[error("Missing access secret. Set ZHIHU_ACCESS_SECRET or run 'zhihu auth set-secret'.")]
    MissingSecret,

    #[error("Configuration directory could not be determined")]
    ConfigDirNotFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse config: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("Failed to serialize config: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("HTTP {status}: {body}")]
    Api { status: StatusCode, body: String },

    #[error("Non-JSON response from API")]
    NonJsonResponse,

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

pub type Result<T> = std::result::Result<T, ZhihuError>;
```

- [ ] **Step 2: 在 main.rs 中注册模块**

Modify `src/main.rs`:

```rust
mod error;

fn main() {
    println!("Hello, world!");
}
```

- [ ] **Step 3: 编译检查**

Run: `cargo check`
Expected: 通过

- [ ] **Step 4: 提交**

```bash
git add src/error.rs src/main.rs
git commit -m "feat: add unified error type

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 3: 配置与凭证解析

**Files:**
- Create: `src/config.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 编写配置模块**

```rust
use crate::error::{Result, ZhihuError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub access_secret: Option<String>,
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        dirs::config_dir()
            .map(|d| d.join("zhihu-cli"))
            .ok_or(ZhihuError::ConfigDirNotFound)
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> Result<Config> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        let path = dir.join("config.toml");
        fs::create_dir_all(&dir)?;
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }
        Ok(())
    }

    pub fn resolve_secret() -> Result<String> {
        if let Ok(secret) = std::env::var("ZHIHU_ACCESS_SECRET") {
            let secret = secret.trim().to_string();
            if !secret.is_empty() {
                return Ok(secret);
            }
        }
        if let Some(secret) = Config::load()?.access_secret {
            let secret = secret.trim().to_string();
            if !secret.is_empty() {
                return Ok(secret);
            }
        }
        Err(ZhihuError::MissingSecret)
    }

    pub fn set_secret(secret: String) -> Result<()> {
        let mut config = Config::load()?;
        config.access_secret = Some(secret);
        config.save()
    }
}
```

- [ ] **Step 2: 注册模块**

Modify `src/main.rs`:

```rust
mod config;
mod error;
```

- [ ] **Step 3: 编译检查**

Run: `cargo check`
Expected: 通过

- [ ] **Step 4: 提交**

```bash
git add src/config.rs src/main.rs
git commit -m "feat: add config loading and secret resolution

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 4: JSON 输出辅助

**Files:**
- Create: `src/output.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 编写输出模块**

```rust
use crate::error::ZhihuError;
use serde::Serialize;
use std::process;

#[derive(Debug, Serialize)]
struct ErrorOutput {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<i32>,
}

pub fn print_json<T: Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{}", s),
        Err(e) => print_error(&ZhihuError::InvalidArgument(format!("JSON serialize failed: {e}")),
        ),
    }
}

pub fn print_json_line<T: Serialize>(value: &T) {
    match serde_json::to_string(value) {
        Ok(s) => println!("{}", s),
        Err(e) => print_error(
            &ZhihuError::InvalidArgument(format!("JSON serialize failed: {e}")),
        ),
    }
}

pub fn print_error(err: &ZhihuError) -> ! {
    let code = match err {
        ZhihuError::MissingSecret => Some(20001),
        _ => None,
    };
    let out = ErrorOutput {
        error: err.to_string(),
        code,
    };
    eprintln!("{}", serde_json::to_string(&out).unwrap_or_else(|_| {
        r#"{"error":"Failed to serialize error"}"#.to_string()
    }));
    process::exit(1);
}
```

- [ ] **Step 2: 注册模块**

Modify `src/main.rs`:

```rust
mod config;
mod error;
mod output;
```

- [ ] **Step 3: 编译检查**

Run: `cargo check`
Expected: 通过

- [ ] **Step 4: 提交**

```bash
git add src/output.rs src/main.rs
git commit -m "feat: add JSON output helpers

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 5: HTTP 客户端

**Files:**
- Create: `src/client.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 编写客户端模块**

```rust
use crate::config::Config;
use crate::error::{Result, ZhihuError};
use reqwest::{Client, Method, RequestBuilder, StatusCode};
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

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
```

- [ ] **Step 2: 注册模块**

Modify `src/main.rs`:

```rust
mod client;
mod config;
mod error;
mod output;
```

- [ ] **Step 3: 编译检查**

Run: `cargo check`
Expected: 通过

- [ ] **Step 4: 提交**

```bash
git add src/client.rs src/main.rs
git commit -m "feat: add authenticated HTTP client

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 6: CLI 参数解析

**Files:**
- Create: `src/cli.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 编写 CLI 派生结构**

```rust
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "zhihu", version, about = "Zhihu Open Platform CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Authentication commands
    Auth {
        #[command(subcommand)]
        subcommand: AuthCommand,
    },
    /// Search commands
    Search {
        #[command(subcommand)]
        subcommand: SearchCommand,
    },
    /// Ask Zhida
    Ask(AskArgs),
}

#[derive(Debug, clap::Args)]
pub struct AskArgs {
    /// User query
    pub query: String,
    /// Model tier
    #[arg(long, value_enum, default_value = "thinking")]
    pub model: ModelTier,
    /// Stream output
    #[arg(long)]
    pub stream: bool,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Interactive login
    Login,
    /// Set access secret directly
    SetSecret {
        /// Access secret
        secret: String,
    },
    /// Show authentication status
    Status,
}

#[derive(Debug, Subcommand)]
pub enum SearchCommand {
    /// Search within Zhihu
    Zhihu {
        /// Search query
        query: String,
        /// Number of results
        #[arg(long, default_value = "10")]
        count: i32,
    },
    /// Search the whole web
    Global {
        /// Search query
        query: String,
        /// Number of results
        #[arg(long, default_value = "10")]
        count: i32,
        /// Advanced filter expression
        #[arg(long)]
        filter: Option<String>,
        /// Index database
        #[arg(long, value_enum, default_value = "all")]
        db: SearchDb,
    },
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum ModelTier {
    #[default]
    Fast,
    Thinking,
    Agent,
}

impl ModelTier {
    pub fn api_name(&self) -> &'static str {
        match self {
            ModelTier::Fast => "zhida-fast-1p5",
            ModelTier::Thinking => "zhida-thinking-1p5",
            ModelTier::Agent => "zhida-agent",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum SearchDb {
    #[default]
    All,
    Realtime,
    Static,
}

impl SearchDb {
    pub fn api_name(&self) -> &'static str {
        match self {
            SearchDb::All => "all",
            SearchDb::Realtime => "realtime",
            SearchDb::Static => "static",
        }
    }
}
```

- [ ] **Step 2: 注册模块并改写 main**

Replace `src/main.rs` with:

```rust
mod cli;
mod client;
mod config;
mod error;
mod output;

use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    println!("{:?}", cli.command);
}
```

- [ ] **Step 3: 编译并测试 help**

Run: `cargo run -- --help`
Expected: 显示命令帮助

- [ ] **Step 4: 提交**

```bash
git add src/cli.rs src/main.rs
git commit -m "feat: add CLI argument parsing

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 7: Auth 命令实现

**Files:**
- Create: `src/commands/mod.rs`
- Create: `src/commands/auth.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 创建 commands 模块**

`src/commands/mod.rs`:

```rust
pub mod auth;
pub mod ask;
pub mod search;
```

- [ ] **Step 2: 实现 auth 命令**

`src/commands/auth.rs`:

```rust
use crate::cli::AuthCommand;
use crate::config::Config;
use crate::error::Result;
use crate::output::{print_error, print_json};
use serde_json::json;
use std::io::{self, Write};

pub async fn run(cmd: AuthCommand) {
    match handle(cmd).await {
        Ok(value) => print_json(&value),
        Err(e) => print_error(&e),
    }
}

async fn handle(cmd: AuthCommand) -> Result<serde_json::Value> {
    match cmd {
        AuthCommand::Login => {
            print!("Enter access secret: ");
            io::stdout().flush().unwrap();
            let mut secret = String::new();
            io::stdin().read_line(&mut secret)?;
            let secret = secret.trim().to_string();
            if secret.is_empty() {
                return Err(crate::error::ZhihuError::InvalidArgument(
                    "secret cannot be empty".into(),
                ));
            }
            Config::set_secret(secret)?;
            Ok(json!({"status":"ok","message":"secret saved"}))
        }
        AuthCommand::SetSecret { secret } => {
            let secret = secret.trim().to_string();
            if secret.is_empty() {
                return Err(crate::error::ZhihuError::InvalidArgument(
                    "secret cannot be empty".into(),
                ));
            }
            Config::set_secret(secret)?;
            Ok(json!({"status":"ok","message":"secret saved"}))
        }
        AuthCommand::Status => {
            let config = Config::load()?;
            let configured = config.access_secret.is_some()
                || std::env::var("ZHIHU_ACCESS_SECRET").is_ok();
            let source = if std::env::var("ZHIHU_ACCESS_SECRET").is_ok() {
                "env"
            } else if config.access_secret.is_some() {
                "config"
            } else {
                "none"
            };
            Ok(json!({
                "configured": configured,
                "source": source,
            }))
        }
    }
}
```

- [ ] **Step 3: 注册模块并连接 main**

Modify `src/main.rs`:

```rust
mod cli;
mod client;
mod commands;
mod config;
mod error;
mod output;

use clap::Parser;
use cli::{Cli, Command};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Auth { subcommand } => commands::auth::run(subcommand).await,
        Command::Search { .. } => {}
        Command::Ask(_) => {}
    }
}
```

- [ ] **Step 4: 编译并测试 status**

Run: `cargo run -- auth status`
Expected: `{"configured":false,"source":"none"}`

- [ ] **Step 5: 提交**

```bash
git add src/commands/mod.rs src/commands/auth.rs src/main.rs
git commit -m "feat: implement auth commands

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 8: Search 命令实现

**Files:**
- Create: `src/commands/search.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 实现 search 命令**

`src/commands/search.rs`:

```rust
use crate::cli::{SearchCommand, SearchDb};
use crate::client::ZhihuClient;
use crate::error::Result;
use crate::output::{print_error, print_json};

pub async fn run(cmd: SearchCommand) {
    match handle(cmd).await {
        Ok(value) => print_json(&value),
        Err(e) => print_error(&e),
    }
}

async fn handle(cmd: SearchCommand) -> Result<serde_json::Value> {
    let client = ZhihuClient::new()?;
    match cmd {
        SearchCommand::Zhihu { query, count } => {
            let count = count.clamp(1, 10).to_string();
            client
                .get(
                    "/api/v1/content/zhihu_search",
                    &[("Query", &query), ("Count", &count)],
                )
                .await
        }
        SearchCommand::Global {
            query,
            count,
            filter,
            db,
        } => {
            let count = count.clamp(1, 20).to_string();
            let mut params: Vec<(&str, &str)> = vec![("Query", &query), ("Count", &count)];
            let db_str = db.api_name().to_string();
            params.push(("SearchDB", &db_str));
            if let Some(filter) = &filter {
                params.push(("Filter", filter.as_str()));
            }
            client.get("/api/v1/content/global_search", &params).await
        }
    }
}
```

- [ ] **Step 2: 连接 main**

Modify `src/main.rs` 的 match：

```rust
match cli.command {
    Command::Auth { subcommand } => commands::auth::run(subcommand).await,
    Command::Search { subcommand } => commands::search::run(subcommand).await,
    Command::Ask { .. } => {}
}
```

- [ ] **Step 3: 编译检查**

Run: `cargo check`
Expected: 通过

- [ ] **Step 4: 提交**

```bash
git add src/commands/search.rs src/main.rs
git commit -m "feat: implement search commands

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 9: Ask 命令（非流式）

**Files:**
- Create: `src/commands/ask.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 实现 ask 非流式**

`src/commands/ask.rs`:

```rust
use crate::cli::{AskArgs, ModelTier};
use crate::client::ZhihuClient;
use crate::error::Result;
use crate::output::{print_error, print_json};
use serde_json::{json, Value};

pub async fn run(args: AskArgs) {
    match handle(args).await {
        Ok(value) => print_json(&value),
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
        return Err(crate::error::ZhihuError::Api {
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
                            crate::output::print_json_line(&json!({"delta": delta}),
                            );
                        }
                        if let Some(reason) = finish_reason {
                            crate::output::print_json_line(
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
```

- [ ] **Step 2: 连接 main**

Modify `src/main.rs`：

```rust
match cli.command {
    Command::Auth { subcommand } => commands::auth::run(subcommand).await,
    Command::Search { subcommand } => commands::search::run(subcommand).await,
    Command::Ask(args) => commands::ask::run(args).await,
}
```

- [ ] **Step 3: 编译检查**

Run: `cargo check`
Expected: 通过

- [ ] **Step 4: 提交**

```bash
git add src/commands/ask.rs src/main.rs
git commit -m "feat: implement ask command with streaming

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 10: 完善入口与端到端测试

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: 完善 main 错误兜底**

确保 `src/main.rs` 最终形态：

```rust
mod cli;
mod client;
mod commands;
mod config;
mod error;
mod output;

use clap::Parser;
use cli::{Cli, Command};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Auth { subcommand } => commands::auth::run(subcommand).await,
        Command::Search { subcommand } => commands::search::run(subcommand).await,
        Command::Ask(args) => commands::ask::run(args).await,
    }
}
```

- [ ] **Step 2: 手动验证 help**

Run: `cargo run -- --help`
Expected: 展示完整命令帮助

- [ ] **Step 3: 手动验证 auth status**

Run: `cargo run -- auth status`
Expected: `{"configured":false,"source":"none"}`

- [ ] **Step 4: 手动验证 set-secret**

Run: `cargo run -- auth set-secret test-secret && cargo run -- auth status`
Expected: 先显示 saved，再显示 `{"configured":true,"source":"config"}`

- [ ] **Step 5: 清理测试配置**

Run: `rm -rf ~/.zhihu-cli`

- [ ] **Step 6: 提交**

```bash
git add src/main.rs
git commit -m "chore: wire up main dispatch

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 11: 添加单元测试

**Files:**
- Create: `src/config.rs` tests (inline)
- Create: `tests/cli_tests.rs` (optional集成测试)

- [ ] **Step 1: 为 config 添加单元测试**

在 `src/config.rs` 末尾追加：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_serde_roundtrip() {
        let config = Config {
            access_secret: Some("secret".into()),
        };
        let s = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&s).unwrap();
        assert_eq!(parsed.access_secret, Some("secret".into()));
    }

    #[test]
    fn empty_config_is_default() {
        let s = "";
        let config: Config = toml::from_str(s).unwrap();
        assert!(config.access_secret.is_none());
    }
}
```

- [ ] **Step 2: 运行测试**

Run: `cargo test`
Expected: 通过

- [ ] **Step 3: 提交**

```bash
git add src/config.rs
git commit -m "test: add config unit tests

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## 自检清单

- [x] Spec coverage: 所有设计文档中的命令（auth、search zhihu/global、ask）都有对应 Task。
- [x] Placeholder scan: 无 TBD/TODO/"implement later"/"add appropriate error handling"。
- [x] Type consistency: `ModelTier`、`SearchDb`、`AskArgs` 在 Task 6 和 Task 9 中保持一致。
- [x] DRY: 鉴权头生成集中在 `client.rs`；JSON 输出集中在 `output.rs`；凭证解析集中在 `config.rs`。
- [x] YAGNI: 不实现分页、不实现 `--table`、不实现多轮 messages（第一版仅支持单条 query）。

---

## 执行方式选择

Plan complete and saved to `docs/superpowers/plans/2026-06-26-zhihu-cli-implementation.md`. Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints.

Which approach do you want?
