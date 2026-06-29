# `zhihu hot` — Zhihu Hot List Command Design

## Summary

Add a new top-level command `zhihu hot` that fetches the current Zhihu hot list via the official Open Platform API and prints the raw JSON response.

## Motivation

The CLI currently supports `auth`, `search`, and `ask`, but does not expose the documented `hot_list` endpoint. Adding this command completes the surface area for content discovery without requiring the user to construct raw `curl` requests.

## API Reference

- **Endpoint**: `GET https://developer.zhihu.com/api/v1/content/hot_list`
- **Headers**:
  - `Authorization: Bearer <access_secret>`
  - `X-Request-Timestamp: <unix_seconds>`
  - `Content-Type: application/json`
- **Query parameter**: `Limit` (Int32, optional)
  - Default: `30`
  - Server clamps `Limit <= 0` or `Limit > 30` back to `30`.
- **Response shape**:
  ```json
  {
    "Code": 0,
    "Message": "success",
    "Data": {
      "Total": 2,
      "Items": [
        {
          "Title": "...",
          "Url": "...",
          "ThumbnailUrl": "...",
          "Summary": "..."
        }
      ]
    }
  }
  ```

## CLI Surface

```
zhihu hot [OPTIONS]
```

- `--limit <i32>` — number of results to request (default: 30, clamped to [1, 30] on the CLI side, matching the API cap).

The command is intentionally minimal because the API takes no filters or search terms.

## Architecture

Follow the established command-module pattern used by `commands/search.rs`:

1. **`src/cli.rs`**
   - Add `Hot(HotArgs)` to the `Command` enum.
   - Add `HotArgs` struct with `#[arg(long)] limit: i32` and default `30`.

2. **`src/commands/hot.rs`** (new file)
   - `pub async fn run(args: HotArgs)` — thin I/O wrapper that prints errors.
   - `async fn handle(args: HotArgs) -> Result<serde_json::Value>` — production path using `ZhihuClient::new()`.
   - `pub(crate) async fn handle_with_client(args: HotArgs, client: &ZhihuClient) -> Result<serde_json::Value>` — testable core.
   - `pub(crate) fn build_request(args: &HotArgs) -> HotRequest` — pure parameter assembly returning the endpoint path and the clamped `Limit` query pair.

3. **`src/main.rs`**
   - Add `Command::Hot(args) => commands::hot::run(args).await`.

4. **`src/commands/mod.rs`**
   - Add `pub mod hot;` alongside `auth`, `ask`, and `search`.

## Output

Pretty-print the raw API response JSON using `output::print_json`, identical to the `search` commands. This keeps the CLI consistent and friendly to downstream tooling such as `jq`.

## Error Handling

Reuse existing mechanisms:
- Missing or empty secret → `ZhihuError::MissingSecret`, rendered with `"code": 20001`.
- HTTP non-2xx or malformed response → `ZhihuError::Api`.
- JSON serialization failure → `ZhihuError::InvalidArgument`.

## Testing Strategy (TDD)

All new production code is preceded by a failing test.

### Unit tests in `src/cli.rs`
- Parse `zhihu hot` and assert default `--limit` is 30.
- Parse `zhihu hot --limit 10` and assert the value is captured.
- Parse `zhihu hot --limit 0` and assert it clamps to 1. Note: while the API itself reverts `Limit <= 0` to 30, the CLI follows the project's existing `[1, max]` clamp convention for count-like parameters.

### Unit tests in `src/commands/hot.rs`
- `build_request` returns path `/api/v1/content/hot_list`.
- `build_request` clamps `limit` to `[1, 30]`.
- `handle_with_client` calls the expected endpoint with `Limit` query parameter and returns the mocked JSON body.
- `dispatch_result` propagates serialization errors and input errors as in `search.rs`.

### CLI tests in `tests/cli.rs`
- `hot_without_auth_fails_with_code_20001` — mirrors `search_zhihu_without_auth_fails`.
- `hot_against_mock_server_succeeds` — runs the full binary against a wiremock server and asserts the output contains `"Code": 0`.

### Integration tests in `tests/integration.rs`
- If `ZHIHU_ACCESS_SECRET` is set, call the real API and assert:
  - `Code == 0`
  - `Data.Total` equals `Data.Items` length.
- If the secret is absent, the test short-circuits and passes.

## Open Questions / Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Command name | `zhihu hot` | Short, intuitive, requested by the user. |
| Limit flag name | `--limit` | Matches the API query parameter name exactly, even though existing commands use `--count`. |
| Default limit | `30` | Matches the API default and returns the full list by default. |
| Output format | Raw pretty JSON | Consistent with existing `search` commands; easy to pipe. |
| Client-side clamp | `[1, 30]` | Avoids sending obviously invalid values and documents the API contract in tests. |

## Future Work (out of scope)

- Human-readable table output.
- Caching or offline mode.
- Filtering by item type (question vs. article).
