---
name: zhihu-cli
description: |
  Use this skill whenever the user wants to interact with the Zhihu Open Platform through the `zhihu` CLI.
  This includes searching Zhihu content, performing global web search via Zhihu, using the Zhida chat/completion API,
  configuring credentials, or understanding CLI output. Use it even if the user only says "zhihu", "search zhihu",
  "zhida", "知乎", or mentions the project `zhihu-cli`.
version: 0.1.0
homepage: https://github.com/zhihu-cli/zhihu-cli
metadata:
  tags: [zhihu, search, cli, agent-tool]
---

# zhihu-cli Skill

This skill tells you how to use the `zhihu` command-line tool to call the Zhihu Open Platform APIs.

## Quick reference

- Binary name: `zhihu`
- Default output: raw JSON on stdout, suitable for piping to `jq` or other tools.
- Error output: JSON object on stdout, process exits with non-zero code.
- Authentication: `ZHIHU_ACCESS_SECRET` environment variable, or `~/.zhihu-cli/config.toml`.

## Authentication

The CLI needs a Zhihu Open Platform Access Secret.

### Option 1: environment variable (preferred for agents)

```bash
export ZHIHU_ACCESS_SECRET="<your-secret>"
```

### Option 2: config file

```bash
zhihu auth set-secret "<your-secret>"
```

This writes `~/.zhihu-cli/config.toml`:

```toml
access_secret = "<your-secret>"
```

### Check status

```bash
zhihu auth status
```

## Commands

### Search Zhihu (站内搜索)

```bash
zhihu search zhihu "QUERY" [--count N]
```

- `QUERY`: search keywords (required)
- `--count`: number of results, default 10, max 10

Example:

```bash
zhihu search zhihu "RAG 评测" --count 5
```

### Search the global web (全网搜索)

```bash
zhihu search global "QUERY" [--count N] [--filter FILTER] [--db all|realtime|static]
```

- `QUERY`: search keywords (required)
- `--count`: number of results, default 10, max 20
- `--filter`: advanced filter expression; must be URL-encoded by the CLI automatically
- `--db`: index database choice

Filter syntax examples:

```text
host=="example.com"
host=="example.com" AND publish_time>=1778494631
(host=="example.com" OR host=="news.example.com") AND publish_time>1778494631
```

Note: `host=="zhihu.com"` is not supported; for Zhihu-only content use `zhihu search zhihu`.

Example:

```bash
zhihu search global "人工智能" --count 5 --filter 'host=="example.com"' --db all
```

### Zhida chat/completion (直答)

```bash
zhihu ask "QUERY" [--model fast|thinking|agent] [--stream]
```

- `QUERY`: user message (required); sent as `messages=[{"role":"user","content":"QUERY"}]`
- `--model`: one of
  - `fast` → `zhida-fast-1p5`
  - `thinking` → `zhida-thinking-1p5`
  - `agent` → `zhida-agent`
- `--stream`: enable streaming output (default off)

Example:

```bash
zhihu ask "什么是 RAG？" --model thinking
```

## Output format

### Success

Search commands return the API's raw JSON response (PascalCase fields):

```json
{
  "Code": 0,
  "Message": "success",
  "Data": {
    "HasMore": false,
    "SearchHashId": "...",
    "Items": [...]
  }
}
```

`zhihu ask` without `--stream` returns the OpenAI-style chat completion JSON.

### Streaming

With `--stream`, `zhihu ask` prints newline-delimited JSON (NDJSON) chunks:

```text
{"delta":{"reasoning_content":"先分析..."}}
{"delta":{"content":"RAG 是..."}}
{"finish_reason":"stop"}
```

When calling from an agent, prefer omitting `--stream` to get a single complete JSON object.

### Errors

All errors are emitted as JSON:

```json
{
  "error": "Set ZHIHU_ACCESS_SECRET or run 'zhihu auth set-secret'",
  "code": 20001
}
```

Common error codes:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 10001 | Bad request parameters |
| 20001 | Authentication failed |
| 30001 | Rate limited |
| 90001 | Internal server error |

## Tips for agents

1. Always check `ZHIHU_ACCESS_SECRET` is set before calling non-auth commands.
2. For search, default `--count` is usually enough; use `--count` only when the user asks for more results.
3. For `zhihu ask`, default to `--model thinking` unless the user asks for a quick answer (`fast`) or a complex multi-step task (`agent`).
4. Do not use `global` search with `host=="zhihu.com"`; use `zhihu search zhihu` instead.
5. Parse output as JSON; on non-zero exit code, show the `error` field to the user.

## Examples

Search Zhihu:

```bash
zhihu search zhihu "如何理解 rave 文化" --count 5
```

Global search with filter:

```bash
zhihu search global "ChatGPT" --count 10 --filter 'host=="openai.com"' --db realtime
```

Ask Zhida:

```bash
zhihu ask "总结 RAG 的核心思路" --model agent
```
