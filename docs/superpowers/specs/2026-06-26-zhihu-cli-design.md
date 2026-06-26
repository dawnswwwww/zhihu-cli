# 知乎 CLI 设计文档

- **日期**：2026-06-26
- **状态**：待实现
- **目标**：构建一个 Rust 命令行工具，封装知乎开放平台的搜索与直答接口，并配套提供 agent 可调用的 Skill 包。

## 1. 背景与范围

知乎开放平台提供以下能力：

- `zhihu_search` / `zhihu_search_skill`：知乎站内搜索
- `global_search` / `global_search_skill`：全网搜索
- `zhida` / `zhida_skill`：知乎直答（OpenAI-style chat completions）

本文档只覆盖 **纯 CLI 工具** 的设计。Skill 包的设计将单独使用 `skill-creator` 完成。

## 2. 设计目标

1. 单一二进制 `zhihu`，跨平台运行。
2. 默认输出原始 JSON，方便脚本和 pipe。
3. 鉴权信息可持久化到配置文件，也支持环境变量覆盖。
4. 支持流式直答输出，同时兼容 agent 聚合读取。
5. 错误以 JSON 形式输出，并返回非 0 退出码。

## 3. 命令结构

```text
zhihu --version
zhihu --help

zhihu auth login              # 交互式输入 access secret
zhihu auth set-secret <SECRET>  # 直接设置 access secret
zhihu auth status             # 显示当前认证状态（是否已配置）

zhihu search zhihu <QUERY> [--count N]
zhihu search global <QUERY> [--count N] [--filter FILTER] [--db all|realtime|static]

zhihu ask <QUERY> [--model fast|thinking|agent] [--stream]
```

### 3.1 子命令说明

| 命令 | 说明 |
|------|------|
| `auth login` | 交互式提示用户输入 secret，写入配置文件 |
| `auth set-secret <SECRET>` | 直接设置 secret，适合脚本或环境变量未设置时 |
| `auth status` | 输出当前是否已配置 secret（不显示完整 secret） |
| `search zhihu` | 调用 `GET /api/v1/content/zhihu_search` |
| `search global` | 调用 `GET /api/v1/content/global_search` |
| `ask` | 调用 `POST /v1/chat/completions` |

## 4. 鉴权设计

### 4.1 凭证来源（按优先级从高到低）

1. 环境变量 `ZHIHU_ACCESS_SECRET`
2. 配置文件 `~/.zhihu-cli/config.toml` 中的 `access_secret`
3. 未配置时返回鉴权错误

### 4.2 配置文件

路径：`~/.zhihu-cli/config.toml`

```toml
access_secret = "<your_access_secret>"
```

- 文件权限建议设置为 `0o600`（仅所有者可读写）。
- 环境变量 `ZHIHU_OPENAPI_BASE_URL` 可用于覆盖默认域名（默认 `https://developer.zhihu.com`），仅影响 CLI 内部调用的 API endpoint。

### 4.3 请求头

每个请求自动附加：

| Header | 值 |
|--------|-----|
| `Authorization` | `Bearer <secret>` |
| `X-Request-Timestamp` | 当前秒级 Unix 时间戳 |
| `Content-Type` | `application/json`（仅 POST） |

## 5. 输出格式

### 5.1 默认输出

默认输出 **原始 JSON**，与 API 响应结构保持一致（搜索接口为 PascalCase，直答接口为 OpenAI-style）。

### 5.2 未来扩展

后续可添加 `--table` 等人可读格式，但第一版不实现。

### 5.3 错误输出

统一输出 JSON：

```json
{
  "error": "Set ZHIHU_ACCESS_SECRET or run 'zhihu auth set-secret'",
  "code": 20001
}
```

- `code` 尽量映射知乎接口错误码；CLI 自身错误使用自定义负值或省略。
- 退出码非 0。

## 6. 流式输出

### 6.1 人类使用

```bash
zhihu ask "解释 RAG" --model thinking --stream
```

输出 **NDJSON**（每行一个 JSON 对象），每个对象对应一个 SSE chunk 的 `delta`：

```text
{"delta":{"reasoning_content":"先分析..."}}
{"delta":{"content":"RAG 是..."}}
{"finish_reason":"stop"}
```

### 6.2 Agent 使用

Agent 通过 shell/pipe 调用时，通常希望拿到完整结果而非流式片段。因此：

- 默认 `stream=false`，一次性输出完整 JSON。
- Agent 显式传 `--stream` 时才进入 NDJSON 模式。

## 7. 请求参数映射

### 7.1 `search zhihu`

| CLI 参数 | API 参数 | 说明 |
|----------|----------|------|
| `QUERY` | `Query` | 必填 |
| `--count` | `Count` | 默认 10，最大 10 |

### 7.2 `search global`

| CLI 参数 | API 参数 | 说明 |
|----------|----------|------|
| `QUERY` | `Query` | 必填 |
| `--count` | `Count` | 默认 10，最大 20 |
| `--filter` | `Filter` | 高级语法筛选，自动 URL 编码 |
| `--db` | `SearchDB` | `all` / `realtime` / `static` |

### 7.3 `ask`

| CLI 参数 | API 参数 | 说明 |
|----------|----------|------|
| `QUERY` | `messages[0].content` | 必填，role 固定为 `user` |
| `--model` | `model` | `zhida-fast-1p5` / `zhida-thinking-1p5` / `zhida-agent` |
| `--stream` | `stream` | 默认 `false` |

## 8. 错误处理

### 8.1 CLI 自身错误

| 场景 | 输出 | 退出码 |
|------|------|--------|
| 缺少 secret | `{"error":"...","code":20001}` | 1 |
| 参数校验失败 | `{"error":"..."}` | 1 |
| 网络超时 | `{"error":"HTTP request timed out"}` | 1 |
| 非 2xx 响应 | `{"error":"HTTP 403","body":"..."}` | 1 |
| 非 JSON 响应 | `{"error":"Non-JSON response"}` | 1 |

### 8.2 API 错误码映射

| 错误码 | 含义 |
|--------|------|
| 0 | 成功 |
| 10001 | 参数错误 |
| 20001 | 鉴权失败 |
| 30001 | 频率限制 |
| 90001 | 内部错误 |

## 9. 技术栈

- 语言：Rust
- CLI 解析：`clap`
- HTTP 客户端：`reqwest`
- 配置管理：`toml` + `dirs`
- 流式 SSE 解析：手动实现或轻量库

## 10. 里程碑

### M1：基础骨架

- [ ] 项目初始化（Cargo、依赖、基本目录结构）
- [ ] `auth login` / `set-secret` / `status`
- [ ] 配置文件读写

### M2：搜索接口

- [ ] `search zhihu`
- [ ] `search global`
- [ ] 错误处理与 JSON 输出

### M3：直答接口

- [ ] `ask` 非流式
- [ ] `ask --stream` NDJSON

### M4：Skill 包

- [ ] 使用 `skill-creator` 设计并生成 agent 可调用的 Skill 包

## 11. 待确认事项

- Skill 包的具体调用约定（待 `skill-creator` 设计）。
- 是否需要支持多个 base URL 配置项（如 `ZHIHU_OPENAPI_BASE_URL`）。
- 是否需要为 `ask` 支持多轮 `messages` 输入（第一版仅支持单条 query）。
