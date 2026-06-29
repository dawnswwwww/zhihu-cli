# 知乎开放平台 API 参考（CLI 设计用）

> 来源：https://developer.zhihu.com/docs
> 抓取时间：2026-06-26
> 接口统一接入域名：`https://developer.zhihu.com`

## 1. 全局约定

### 1.1 鉴权方式

所有数据接口统一使用 **Bearer Token** 鉴权。

| Header | 示例值 | 说明 |
|--------|--------|------|
| `Authorization` | `Bearer <your_access_secret>` | 在个人中心获取 Access Secret |
| `X-Request-Timestamp` | `1742822400` | **秒级** Unix 时间戳，服务端会校验 |
| `Content-Type` | `application/json` | JSON 接口固定值 |

### 1.2 通用错误码

| 错误码 | 说明 |
|--------|------|
| 0 | 成功 |
| 10001 | 参数错误 |
| 20001 | 鉴权失败 |
| 30001 | 频率限制 |
| 90001 | 内部错误 |

### 1.3 Skill 与 API 的关系

- **API**：原始 HTTP 接口，字段为 PascalCase，返回完整数据。
- **Skill**：面向 AI 助手 / Agent 的封装包，字段为 snake_case，输出更精简。
- Skill 提供 zip 下载，CLI 如果只做 HTTP 调用可暂不处理 Skill 包；若要做本地封装，可下载 zip 解析。

---

## 2. 知乎热榜（hot_list）

### 2.1 API：`hot_list`

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/content/hot_list` |
| 说明 | 获取当前知乎热榜内容，返回问题/文章列表 |

#### 请求参数

| Header | 示例值 | 说明 |
|--------|--------|------|
| `Authorization` | `Bearer <your_access_secret>` | Access Secret |
| `X-Request-Timestamp` | `1742822400` | 秒级 Unix 时间戳 |

| Query 参数 | 类型 | 必填 | 说明 |
|------------|------|------|------|
| `Limit` | Int32 | 否 | 默认 30，最大 30；超出范围时服务端回退为 30 |

#### 响应字段

| Data 字段 | 类型 | 说明 |
|-----------|------|------|
| `Total` | Int64 | 实际返回的热榜条数 |
| `Items` | Array[Item] | 热榜内容列表 |

Item 字段：

| 字段 | 类型 | 说明 |
|------|------|------|
| `Title` | String | 热榜标题 |
| `Url` | String | 对应的知乎链接 |
| `ThumbnailUrl` | String | 缩略图 URL，无封面时为空字符串 |
| `Summary` | String | 内容摘要，无摘要时为空字符串 |

#### 响应示例

```json
{
  "Code": 0,
  "Message": "success",
  "Data": {
    "Total": 2,
    "Items": [
      {
        "Title": "如何评价某个热点问题？",
        "Url": "https://www.zhihu.com/question/123456789",
        "ThumbnailUrl": "https://pic1.zhimg.com/...jpg",
        "Summary": "这是该问题的内容摘要"
      },
      {
        "Title": "一篇正在热榜上的文章标题",
        "Url": "https://zhuanlan.zhihu.com/p/987654321",
        "ThumbnailUrl": "",
        "Summary": ""
      }
    ]
  }
}
```

### 2.2 CLI 用法

```bash
zhihu hot [--limit N]
```

- `--limit` 默认 30，CLI 会将其限制在 `[1, 30]`。

---

## 3. 知乎搜索（站内搜索）

### 2.1 API：`zhihu_search`

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/content/zhihu_search` |
| 说明 | 知乎站内内容搜索，返回问题、回答或文章 |

#### 请求参数

| Query 参数 | 类型 | 必填 | 说明 |
|------------|------|------|------|
| `Query` | String | 是 | 查询关键词，不能为空 |
| `Count` | Int32 | 否 | 默认 10，最大 10；超出自动截断，<=0 回退为 10 |

#### 响应字段

| Data 字段 | 类型 | 说明 |
|-----------|------|------|
| `HasMore` | Bool | 当前固定返回 `false` |
| `SearchHashId` | String | 搜索请求标识 |
| `Items` | Array[Item] | 搜索结果 |
| `EmptyReason` | String | 无结果原因 |

Item 字段：

| 字段 | 类型 | 说明 |
|------|------|------|
| `Title` | String | 标题 |
| `ContentType` | String | `Article` / `Answer` 等 |
| `ContentID` | String | 内容标识 |
| `ContentText` | String | 内容摘要 |
| `Url` | String | 带 `utm_medium=openapi_platform` 溯源参数 |
| `CommentCount` | Int32 | 评论数 |
| `VoteUpCount` | Int32 | 赞同数 |
| `AuthorName` | String | 作者昵称 |
| `AuthorAvatar` | String | 作者头像 |
| `AuthorBadge` / `AuthorBadgeText` | String | 认证图标 / 文案 |
| `EditTime` | Int32 | 发布时间或更新时间戳 |
| `CommentInfoList` | Array | 精选评论（可选） |
| `AuthorityLevel` | String | 权威等级 |
| `RankingScore` | Float32 | 排序分数 |

### 2.2 Skill：`zhihu_search_skill`

| 项目 | 值 |
|------|-----|
| 下载 | `https://developer.zhihu.com/download/zhihu_search_skills.zip` |
| 输入 | `query`（必填）、`count`（默认 10，最大 10） |
| 输出 | `code`, `message`, `item_count`, `items`（含 `title`, `summary`, `url`, `author_name`, `vote_up_count`, `comment_count`, `edit_time`） |

---

## 4. 全网搜索

### 4.1 API：`global_search`

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/content/global_search` |
| 说明 | 全网内容搜索，可筛选站点与发布时间 |

#### 请求参数

| Query 参数 | 类型 | 必填 | 说明 |
|------------|------|------|------|
| `Query` | String | 是 | 查询关键词 |
| `Count` | Int32 | 否 | 默认 10，最大 20 |
| `Filter` | String | 否 | 高级语法筛选表达式，需 URL 编码 |
| `SearchDB` | String | 否 | 索引库：`all`（默认）、`realtime`、`static` |

#### Filter 高级语法

- `host`: 站点域名，支持 `==`、`!=`，字符串用双引号。
  - 注意：`host=="zhihu.com"` 及其子域名**不支持**，站内搜索请用 `zhihu_search`。
- `publish_time`: 秒级时间戳，支持 `==`、`!=`、`>`、`>=`、`<`、`<=`，数字不用引号。
- 逻辑符：`AND`、`OR`（必须大写），`AND` 优先级高于 `OR`，可用 `()` 控制优先级。

示例：

```text
host=="example.com"
host=="example.com" AND publish_time>=1778494631
(host=="example.com" OR host=="news.example.com") AND publish_time>1778494631
```

#### 响应字段

与 `zhihu_search` 基本一致，额外注意：

- `HasMore` 为真实分页标识（非固定 false）。
- `AuthorityLevel` 含义：`1` 低权威、`2` 中权威、`3` 高权威、`4` 超高权威。
- `ContentText` 中高亮部分使用 `<em>` 标签。

### 4.2 Skill：`global_search_skill`

| 项目 | 值 |
|------|-----|
| 下载 | `https://developer.zhihu.com/download/global_search_skills.zip` |
| 输入 | `query`（必填）、`count`（默认 10）、`filter`、 `search_db`（`all`/`realtime`/`static`） |
| 输出 | 与 `zhihu_search_skill` 结构相同，Item 含 `title`, `summary`, `url`, `author_name`, `edit_time` |

---

## 5. 直答（对话 / 生成）

### 5.1 API：`zhida`

| 项目 | 值 |
|------|-----|
| URL | `POST https://developer.zhihu.com/v1/chat/completions` |
| 说明 | 知乎直答，提供快速回答、深度思考、智能思考三个档位 |
| 响应格式 | `application/json`（非流式） / `text/event-stream`（流式） |

#### 请求体

| Body 字段 | 类型 | 必填 | 说明 |
|-----------|------|------|------|
| `model` | String | 是 | `zhida-fast-1p5` / `zhida-thinking-1p5` / `zhida-agent` |
| `messages` | Array[Message] | 是 | 对话消息列表 |
| `stream` | Bool | 否 | 默认 `false` |

Message：

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `role` | String | 是 | 消息角色 |
| `content` | String | 是 | 问题内容 |

#### 非流式响应

```json
{
  "id": "chatcmpl-xxxx",
  "object": "chat.completion",
  "created": 1740470400,
  "model": "zhida-thinking-1p5",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "reasoning_content": "先给出分析过程...",
        "content": "..."
      },
      "finish_reason": "stop"
    }
  ]
}
```

#### 流式响应

SSE 格式，每条以 `data: ` 开头，结束为 `data: [DONE]`；中间可能夹杂心跳注释 `: keep-alive`。

### 5.2 Skill：`zhida_skill`

| 项目 | 值 |
|------|-----|
| 下载 | `https://developer.zhihu.com/download/zhida_skills.zip` |
| 简化输入 | `query`（必填）、`model`（必填）、`stream`（默认 false） |
| 对话式输入 | `model`、`messages`、`stream` |
| 输出 | `code`, `id`, `model`, `stream`, `content`, `reasoning_content`, `finish_reason` |

---

## 6. CLI 设计初步建议

### 6.1 命令分层

```text
zhihu auth login --secret <ACCESS_SECRET>     # 保存 secret 并自动生成时间戳
zhihu search zhihu <QUERY> [--count N]
zhihu search global <QUERY> [--count N] [--filter ...] [--db all|realtime|static]
zhihu hot [--limit N]
zhihu ask <QUERY> [--model fast|thinking|agent] [--stream]
```

### 6.2 需要提前确认的设计点

1. **鉴权配置存储**：Access Secret 是写入本地配置文件（如 `~/.zhihu-cli/config.toml`），还是每次通过环境变量 `ZHIHU_ACCESS_SECRET` 传入？
2. **输出格式**：默认输出可读表格，还是 JSON？是否提供 `--json` / `--table` 开关？
3. **分页策略**：`global_search` 支持 `HasMore`，CLI 是否需要内置分页（`--page` / `--offset`）？当前文档未提供 offset 参数，需要实测或追问。
4. **流式输出**：`zhida` 流式响应如何与终端交互（是否逐字打印、是否支持 `--no-stream` 聚合）？
5. **Skill 包支持**：CLI 是否只需调用 HTTP API，还是也要能下载 / 执行本地 Skill zip？

### 6.3 实现注意事项

- 每个请求必须带 `X-Request-Timestamp`，建议用当前 Unix 秒级时间戳自动生成。
- `global_search` 的 `Filter` 需要 URL 编码，CLI 应提供参数封装避免用户手写表达式。
- `zhida` 兼容 OpenAI-style 接口语义，便于复用现有的 chat completion 客户端。
- 错误处理应区分：参数错误（10001）、鉴权失败（20001）、频率限制（30001）。
