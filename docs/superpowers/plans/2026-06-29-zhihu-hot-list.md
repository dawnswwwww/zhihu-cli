# `zhihu hot` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `zhihu hot [--limit N]` command that fetches the Zhihu hot list from `/api/v1/content/hot_list` and prints the raw JSON response.

**Architecture:** Follow the existing `search` command pattern: clap-derived args in `src/cli.rs`, a new `src/commands/hot.rs` module with pure `build_request` and injectable `handle_with_client`, dispatch in `src/main.rs`, and wiremock-backed unit + binary-level CLI tests.

**Tech Stack:** Rust 2024, clap, reqwest, serde_json, tokio, wiremock, assert_cmd, serial_test.

---

## File map

| File | Responsibility |
|------|----------------|
| `src/cli.rs` | Add `HotArgs` and `Command::Hot` variant; co-located parse tests. |
| `src/commands/mod.rs` | Declare the new `hot` submodule. |
| `src/commands/hot.rs` | `run`, `handle`, `handle_with_client`, `build_request`, and their tests. |
| `src/main.rs` | Dispatch `Command::Hot` to `commands::hot::run`. |
| `tests/cli.rs` | Binary-level tests: unauthenticated failure and mock-server success. |
| `tests/integration.rs` | Optional real-API round-trip test. |

---

### Task 1: Add CLI args and parse tests

**Files:**
- Modify: `src/cli.rs`
- Test: `src/cli.rs` (co-located `#[cfg(test)] mod tests`)

- [ ] **Step 1: Write the failing tests**

Add these tests inside the existing `mod tests` in `src/cli.rs`:

```rust
#[test]
fn parse_hot_defaults_to_limit_thirty() {
    let cli = Cli::parse_from(["zhihu", "hot"]);
    match cli.command {
        Command::Hot(args) => {
            assert_eq!(args.limit, 30);
        }
        _ => panic!("expected Hot command"),
    }
}

#[test]
fn parse_hot_with_limit() {
    let cli = Cli::parse_from(["zhihu", "hot", "--limit", "10"]);
    match cli.command {
        Command::Hot(args) => {
            assert_eq!(args.limit, 10);
        }
        _ => panic!("expected Hot command"),
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test parse_hot --lib
```

Expected: compile error or test failure because `Command::Hot` and `HotArgs` do not exist.

- [ ] **Step 3: Add `HotArgs` and `Command::Hot`**

Add to `src/cli.rs`:

```rust
#[derive(Debug, clap::Args)]
pub struct HotArgs {
    /// Number of results to return
    #[arg(long, default_value = "30")]
    pub limit: i32,
}
```

Add a new variant to the `Command` enum:

```rust
/// Show Zhihu hot list
Hot(HotArgs),
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test parse_hot --lib
```

Expected: both tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/cli.rs
git commit -m "feat(cli): add zhihu hot argument parsing

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2: Declare the `hot` command module

**Files:**
- Modify: `src/commands/mod.rs`
- Test: compile check only

- [ ] **Step 1: Add module declaration**

In `src/commands/mod.rs`, add:

```rust
pub mod hot;
```

- [ ] **Step 2: Verify the crate compiles**

```bash
cargo check
```

Expected: success (the module is empty or does not exist yet; if `hot.rs` is missing, create an empty `src/commands/hot.rs` first).

- [ ] **Step 3: Commit**

```bash
# create an empty file if it does not exist
touch src/commands/hot.rs
git add src/commands/mod.rs src/commands/hot.rs
git commit -m "chore: declare hot command module

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 3: Implement `build_request` and its tests

**Files:**
- Create/Modify: `src/commands/hot.rs`
- Test: `src/commands/hot.rs` (co-located tests)

- [ ] **Step 1: Write the failing tests**

Replace the empty `src/commands/hot.rs` with the test module first:

```rust
#[cfg(test)]
mod tests {
    use super::build_request;
    use crate::cli::HotArgs;

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
        let limit = pairs(&req).iter().find(|(k, _)| *k == "Limit").unwrap();
        assert_eq!(limit.1, "30");
    }

    #[test]
    fn hot_request_limit_clamps_to_one_through_thirty() {
        let make = |limit: i32| HotArgs { limit };
        let assert_limit = |limit: i32, expected: &str| {
            let req = build_request(&make(limit));
            let pair = pairs(&req).iter().find(|(k, _)| *k == "Limit").unwrap();
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
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test hot_request --lib
```

Expected: compile errors because `HotArgs`, `HotRequest`, and `build_request` are not defined in this module.

- [ ] **Step 3: Implement `build_request`**

Add to the top of `src/commands/hot.rs`:

```rust
use crate::cli::HotArgs;

#[derive(Debug, PartialEq)]
pub(crate) struct HotRequest {
    pub path: &'static str,
    pub query: Vec<(&'static str, String)>,
}

pub(crate) fn build_request(args: &HotArgs) -> HotRequest {
    HotRequest {
        path: "/api/v1/content/hot_list",
        query: vec![("Limit", args.limit.clamp(1, 30).to_string())],
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test hot_request --lib
```

Expected: all three tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/commands/hot.rs
git commit -m "feat(hot): add hot_list request builder with clamping

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 4: Implement `handle_with_client`, `run`, and dispatch

**Files:**
- Modify: `src/commands/hot.rs`
- Modify: `src/main.rs`
- Test: `src/commands/hot.rs`

- [ ] **Step 1: Write the failing test for the HTTP layer**

Add this test inside `src/commands/hot.rs` `mod tests`:

```rust
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
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test handle_with_client_calls_hot_list --lib
```

Expected: compile error because `handle_with_client` does not exist.

- [ ] **Step 3: Implement the command handlers and dispatch**

Add to `src/commands/hot.rs`:

```rust
use crate::client::ZhihuClient;
use crate::error::Result;
use crate::output::{print_error, print_json};
use serde::Serialize;

pub async fn run(args: HotArgs) {
    if let Err(e) = dispatch_result(handle(args).await) {
        print_error(&e);
    }
}

pub(crate) fn dispatch_result<T: Serialize>(result: Result<T>) -> Result<()> {
    match result {
        Ok(value) => print_json(&value),
        Err(e) => Err(e),
    }
}

async fn handle(args: HotArgs) -> Result<serde_json::Value> {
    handle_with_client(args, &ZhihuClient::new()?).await
}

pub(crate) async fn handle_with_client(args: HotArgs, client: &ZhihuClient) -> Result<serde_json::Value> {
    let req = build_request(&args);
    let query_refs: Vec<(&str, &str)> = req.query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    client.get(req.path, &query_refs).await
}
```

Update `src/main.rs`:

```rust
Command::Hot(args) => zhihu_cli::commands::hot::run(args).await,
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test handle_with_client_calls_hot_list --lib
cargo test dispatch_result --lib
cargo test hot_request --lib
```

Expected: all tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/commands/hot.rs src/main.rs
git commit -m "feat(hot): wire up hot_list HTTP handler and main dispatch

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 5: Add CLI-level regression tests

**Files:**
- Modify: `tests/cli.rs`
- Test: `tests/cli.rs`

- [ ] **Step 1: Add unauthenticated test**

Add to `tests/cli.rs`:

```rust
#[test]
fn hot_without_auth_fails() {
    with_temp_home(|tmp| {
        let mut cmd = Command::cargo_bin("zhihu").unwrap();
        cmd.env("HOME", tmp.path());
        cmd.env_remove("ZHIHU_ACCESS_SECRET");
        cmd.arg("hot");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("\"code\":20001"))
            .stderr(predicate::str::contains("Missing access secret"));
    });
}
```

- [ ] **Step 2: Add mock-server success test**

Add to `tests/cli.rs`:

```rust
#[tokio::test]
#[serial]
async fn cli_hot_against_mock_server_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/content/hot_list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Code": 0,
            "Message": "ok",
            "Data": { "Total": 0, "Items": [] },
        })))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("zhihu").unwrap();
    cmd.env("ZHIHU_ACCESS_SECRET", "fake");
    cmd.env("ZHIHU_OPENAPI_BASE_URL", server.uri());
    cmd.arg("hot").arg("--limit").arg("5");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"Code\": 0"));
}
```

- [ ] **Step 3: Run the new CLI tests**

```bash
cargo test hot_without_auth --test cli
cargo test cli_hot_against_mock_server --test cli
```

Expected: both PASS.

- [ ] **Step 4: Commit**

```bash
git add tests/cli.rs
git commit -m "test(cli): add hot command e2e tests

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 6: Add integration test

**Files:**
- Modify: `tests/integration.rs`
- Test: `tests/integration.rs`

- [ ] **Step 1: Add the real-API test**

Add to `tests/integration.rs`:

```rust
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
    assert!(
        items_len <= total as usize,
        "returned items should not exceed reported total"
    );
}
```

- [ ] **Step 2: Run integration tests**

Without a secret they short-circuit:

```bash
cargo test hot_list_returns_results --test integration
```

Expected: PASS (test returns early).

With a secret:

```bash
ZHIHU_ACCESS_SECRET=your_secret cargo test hot_list_returns_results --test integration
```

Expected: PASS against the real API.

- [ ] **Step 3: Commit**

```bash
git add tests/integration.rs
git commit -m "test(integration): add hot_list real API round-trip

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 7: Full local gate

**Files:** all of the above

- [ ] **Step 1: Run lint**

```bash
make lint
```

Expected: no warnings, no errors.

- [ ] **Step 2: Run tests**

```bash
make test
```

Expected: all tests PASS.

- [ ] **Step 3: Run coverage gate**

```bash
make coverage
```

Expected: total line coverage >= 80%.

- [ ] **Step 4: Run release dry-run**

```bash
make release-dry-run
```

Expected: release binary builds successfully.

- [ ] **Step 5: Commit any fixes**

If any step failed, fix and commit each fix separately. If everything passes, no extra commit is needed.

---

## Self-review checklist

- [ ] Spec coverage: every requirement from `docs/superpowers/specs/2026-06-29-zhihu-hot-list-design.md` maps to a task above.
- [ ] Placeholder scan: no `TBD`, `TODO`, or vague instructions remain.
- [ ] Type consistency: `HotArgs`, `HotRequest`, `build_request`, `handle_with_client` names match across tasks.
- [ ] Test-first: every production change in Tasks 1-4 is preceded by a failing test.
