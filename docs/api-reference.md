# 知乎开放平台 API 参考（CLI 设计用）

> 来源：https://developer.zhihu.com/docs
> 抓取时间：2026-06-26（第 1-4 节）；2026-09-07（第 5-9 节）
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

## 2. 知乎搜索（站内搜索）

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

## 3. 全网搜索

### 3.1 API：`global_search`

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

### 3.2 Skill：`global_search_skill`

| 项目 | 值 |
|------|-----|
| 下载 | `https://developer.zhihu.com/download/global_search_skills.zip` |
| 输入 | `query`（必填）、`count`（默认 10）、`filter`、 `search_db`（`all`/`realtime`/`static`） |
| 输出 | 与 `zhihu_search_skill` 结构相同，Item 含 `title`, `summary`, `url`, `author_name`, `edit_time` |

---

## 4. 直答（对话 / 生成）

### 4.1 API：`zhida`

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

### 4.2 Skill：`zhida_skill`

| 项目 | 值 |
|------|-----|
| 下载 | `https://developer.zhihu.com/download/zhida_skills.zip` |
| 简化输入 | `query`（必填）、`model`（必填）、`stream`（默认 false） |
| 对话式输入 | `model`、`messages`、`stream` |
| 输出 | `code`, `id`, `model`, `stream`, `content`, `reasoning_content`, `finish_reason` |

---

## 5. 热榜

### 5.1 API：`hot_list`

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/content/hot_list` |
| API ID | `hot_list` |
| 说明 | 获取当前知乎热榜，返回结构化的标题、链接、缩略图与摘要列表 |

#### 请求参数

| Query 参数 | 类型 | 必填 | 说明 |
|------------|------|------|------|
| `Limit` | Int32 | 否 | 返回数量，默认 30，最大 30；超出或 <=0 服务端回退为 30 |

#### 响应字段

Data：`Total`（Int64，实际返回条数）、`Items`（数组）。Item：`Title`、`Url`、`ThumbnailUrl`（无封面为空串）、`Summary`（无摘要为空串）。当前仅返回问题和文章两类热榜内容。

#### 错误码

20001 鉴权失败、30001 频率限制、90001 内部错误。

---

## 6. 额度查询

### 6.1 API：`quota`

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/quota` |
| 说明 | 查询当前 Access Secret 所属账号自然日内的各项能力的每日限免额度，不消耗业务额度 |

知识库文件上传/列表/内容列表/检索共用 `knowledge` 额度池；PDF 解析与 PPT 生成共用 `tools` 额度池。

#### 请求参数

| Query 参数 | 类型 | 必填 | 说明 |
|------------|------|------|------|
| `APIIDs` | String | 否 | 逗号分隔的 API ID；不传时返回全部可展示额度 |

可查询额度项：`global_search`（全网搜）、`zhihu_search`（知乎搜索）、`hot_list`（热榜）、`user_data`（知乎用户数据）、`zhida_openai`（直答）、`knowledge`（知识库）、`tools`（小工具）。

#### 响应字段

Data 为数组，元素：`APIID`、`APIName`、`TotalQuota`（int64）、`TotalUsed`（int64）、`RemainingQuota`（int64，最低 0）。

#### 错误码

10001 APIIDs 格式错误或包含未知 ID、20001 鉴权失败、30001 频率限制、90001 读取失败。

---

## 7. 知识库

> 首次使用请先登录直答知识库完成初始化：https://zhida.zhihu.com/repositories/square

### 7.1 API：`knowledge_bases`（知识库列表）

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/knowledge/bases` |
| API ID | `knowledge_bases` |
| 说明 | 获取当前用户创建或订阅的知识库 |

| Query 参数 | 类型 | 必填 | 默认值 | 说明 |
|------------|------|------|--------|------|
| `Scope` | String | 否 | all | `all`、`created` 或 `subscribed` |

响应 Data：`Items`（数组）。KnowledgeBase：`KnowledgeBaseID`、`Name`、`Description`（可选）、`Relation`（created/subscribed/both）、`IsDefault`（Bool）、`Visibility`（private/public）、`ContentCount`（Int64）、`UpdatedAt`（秒级时间戳）。

错误码：10001 参数错误、20001 鉴权失败或无访问权限、30001 频率限制、90001 请求失败。

### 7.2 API：`knowledge_base_items`（知识库内容列表）

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/knowledge/bases/{KnowledgeBaseID}/items` |
| API ID | `knowledge_base_items` |
| 说明 | 分页获取指定知识库中的内容 |

| 参数 | 位置 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|------|--------|------|
| `KnowledgeBaseID` | Path | String | 是 | - | 知识库 ID |
| `Cursor` | Query | String | 否 | 空 | 上一页返回的不透明游标；调用方不要解析或修改 |
| `Limit` | Query | Int32 | 否 | 20 | 每页数量，范围 1..20 |

响应 Data：`Items`、`Total`（Int64）、`HasMore`（Bool）、`NextCursor`（HasMore=true 时返回）。KnowledgeItem：`RecallContentID`、`ContentType`（unknown/file/answer/article）、`Title`（文件名）、`Abstract`（可选）、`CreatedAt`/`UpdatedAt`（秒级时间戳）、`OriginUrl`（原文地址或源文件下载地址）。

请以 `HasMore` 判断是否继续分页，不要根据本页条数推断是否结束。

错误码：10001、20001、30001、40004 知识库不存在、90001。

### 7.3 API：`knowledge_file_upload`（知识库文件上传）

| 项目 | 值 |
|------|-----|
| URL | `POST https://developer.zhihu.com/api/v1/knowledge/files` |
| API ID | `knowledge_file_upload` |
| Content-Type | `multipart/form-data`（不要手工设置带 boundary 的 Content-Type） |
| 说明 | 上传文件并同步完成解析和知识库挂载；不指定知识库时进入默认知识库 |

Form 字段：`File`（文件，必填，单文件最大 100 MB）、`KnowledgeBaseID`（String，可选）。

支持扩展名（忽略大小写）：`pdf, md, txt, ppt, pptx, xlsx, xls, docx, doc, webp, png, jpg, mobi, epub, csv, azw3`。文件名必须合法 UTF-8，不含 NUL/换行/控制字符，清理路径和首尾空白后不超过 255 UTF-8 字节。

响应 Data：`KnowledgeBaseID`、`RecallContentID`、`FileName`（清理后的文件名）、`FileSize`（字节）、`Title`（可选）、`Abstract`（可选）、`OriginUrl`（可选）。

同步接口，较大文件耗时较长；调用方取消请求不代表上传和解析同步取消，不要对超时或未知结果自动重试。相同文件仅在前一次仍处于同步处理阶段时被拦截（40005）。

错误码：10001 文件/文件名/格式/大小或其他参数不合法、20001 鉴权失败或无权上传、30001、40004 知识库不存在、40005 相同文件正在处理中、40006 文件解析失败、90001。

### 7.4 API：`knowledge_search`（知识库检索）

| 项目 | 值 |
|------|-----|
| URL | `POST https://developer.zhihu.com/api/v1/knowledge/search` |
| API ID | `knowledge_search` |
| 说明 | 使用 RAG 从指定知识库或召回范围中检索相关文档片段 |

| Body 字段 | 类型 | 必填 | 默认值 | 说明 |
|-----------|------|------|--------|------|
| `Query` | String | 是 | - | 检索问题，去除首尾空白后不能为空 |
| `KnowledgeBaseIDs` | Array[String] | 条件必填 | `[]` | 知识库 ID 列表 |
| `RecallScopes` | Array[String] | 条件必填 | `[]` | `personal`、`subscription`、`public` |
| `Limit` | Int32 | 否 | 10 | 返回文档数量，范围 1..10 |

`KnowledgeBaseIDs` 与 `RecallScopes` 至少一个非空；同时传入时取并集。

响应 Data：`Items`（按相关性排序）。SearchItem：`Content`（Array[String]，同一文档命中的有序正文片段）、`KnowledgeBaseID`、`DocName`、`RecallContentID`（可选）、`OriginUrl`（可选）。`Limit` 按文档数计算，不按片段数计算。

错误码：10001、20001、30001、50002 检索失败、90001。

---

## 8. 小工具

### 8.1 API：`pdf_parse`（PDF 解析，异步三步）

调用流程：上传 PDF 获取 `file_id`（24 小时内有效）→ 创建解析任务获取 `task_id` → 轮询任务状态 → `succeeded` 后从 `result.url` 下载解析结果。创建任务支持可选 Header `Idempotency-Key`（同 Key 同参数返回同一 task_id；命中幂等重放时响应带 `Idempotent-Replayed: true`）。

#### 8.1.1 上传文件

| 项目 | 值 |
|------|-----|
| URL | `POST https://developer.zhihu.com/resources/v1/files` |
| Content-Type | `multipart/form-data` |

Form：`file`（必填，PDF 文件，最大 100MB；当前仅支持 PDF）。响应 Data：`file_id`（如 `file_00000000fb98...`）。

#### 8.1.2 创建解析任务

| 项目 | 值 |
|------|-----|
| URL | `POST https://developer.zhihu.com/api/v1/pdf-parse/tasks` |
| Content-Type | `application/json` |

Body：`file_id`（String，必填）。响应 Data：`task_id`、`task_status`（初始通常 `pending`）。

#### 8.1.3 查询任务

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/pdf-parse/tasks/{task_id}` |

响应 Data：`task_id`、`task_status`（`pending`/`running`/`succeeded`/`failed`）、`progress`（0..1）、`result`（成功时含 `url` 下载链接、`summary` 可选摘要、`expires_at_ms` 毫秒级过期时间；否则 null）、`error`（失败时含 `code`、`message`；否则 null）。

解析结果 JSON：`schema_version`（v1）、`pages[].page`（从 0 起）、`pages[].blocks[]`（`type` 如 title/text/formula/figure、`box` [x1,y1,x2,y2]、`content` 文本、图片块含 `image.media_type` 与纯 Base64 `image.data`）。调用方应兼容未知 type 和新增字段。

下载链接有效期较短；过期后重新查询任务可获得新链接。

错误码：10001 参数错误、20001 鉴权失败、30001 频率限制、30002 额度不足、40001 幂等键与请求参数冲突、40002 文件不存在/已过期/不可访问、40003 活跃任务数超限、90001 内部错误。

### 8.2 API：`ppt_generation`（PPT 生成，异步两步）

调用流程：提交知乎回答或文章链接创建任务 → 轮询 → `succeeded` 后从 `result.url` 下载 PPTX。支持 `Idempotency-Key`。

#### 8.2.1 创建生成任务

| 项目 | 值 |
|------|-----|
| URL | `POST https://developer.zhihu.com/api/v1/ppt-generation/tasks` |
| Content-Type | `application/json` |

Body：`resource_url`（String，必填，支持 `https://www.zhihu.com/question/{qid}/answer/{aid}`、`https://www.zhihu.com/answer/{aid}`、`https://zhuanlan.zhihu.com/p/{pid}`）、`num_pages`（Int32，必填，范围 6..21）。

#### 8.2.2 查询任务

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/ppt-generation/tasks/{task_id}` |

响应 Data 结构与 pdf_parse 查询一致；成功时 `result` 含 `url`（PPTX 下载链接）与 `expires_at_ms`，无 `summary`。

错误码：10001（含 `resource_url is not supported`）、20001、30001、30002、40001、40003、90001。

---

## 9. 知乎用户数据

开放范围：不提供 OAuth 访问凭证时获取当前调用方本人数据；查看其他用户需先取得该用户的知乎 OAuth 授权，并在请求中提供其 OAuth 访问凭证（Header `X-OAuth-Token`，可选）。

### 9.1 API：`user_contents`（用户的内容）

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/user/contents` |

| Query 参数 | 类型 | 必填 | 说明 |
|------------|------|------|------|
| `ContentType` | String | 是 | `all`、`answer`、`article`、`zvideo`、`pin`、`question` |
| `Offset` | Int64 | 否 | 分页偏移量，默认 0（用 `Paging.NextOffset` 翻页） |
| `Limit` | Int64 | 否 | 返回数量，默认 20，最大 50 |
| `SortField` | String | 否 | `like_count`、`ts`（默认） |
| `SortOrder` | String | 否 | `asc`、`desc`（默认） |

响应 Data：`Items`（ContentItem：`ContentType`、`Url`、`CreatedAt`、`LikeCount`、`CommentCount`、`FavoriteCount`、`Title`、`Summary`）、`Paging`（`IsEnd`、`NextOffset`、`Totals`）。

### 9.2 API：`user_followees`（用户的关注）

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/user/followees` |

Query：`Offset`（默认 0）、`Limit`（默认 20，最大 50）。响应 Data：`Items`（FolloweeItem：`Fullname`、`UrlToken`、`Url`、`AvatarUrl`、`Headline`、`Gender`（0 未知/1 女/2 男）、`FollowerCount`）、`Paging`。

### 9.3 API：`user_collections`（用户的收藏）

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/user/collections` |

Query：`Limit`（默认 20）。响应 Data：`Items`（CollectionContentItem，无 `Paging`）。字段：`ContentType`、`Url`、`CreatedAt`、`FavTime`、`LikeCount`、`CommentCount`、`FavoriteCount`、`Title`、`Summary`、`Favlists`（数组：`UrlToken` Int64、`Title`、`Url`）、`Author`（可选：`Name`、`UrlToken`、`Url`、`Gender`、`Headline`）。

### 9.4 API：`user_favlists`（用户收藏夹列表）

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/user/favlists` |

Query：`Limit`（默认 20）。响应 Data：`Items`（FavlistItem：`UrlToken` Int64（可用于查询收藏夹内容）、`Url`、`Title`、`Description`、`IsPublic`）。

### 9.5 API：`favlist_contents`（收藏夹内容）

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/user/favlist_contents` |

| Query 参数 | 类型 | 必填 | 说明 |
|------------|------|------|------|
| `FavlistUrlToken` | Int64 | 是 | 收藏夹 URL 标识（来自 `user_favlists` 的 `UrlToken`） |
| `Offset` | Int64 | 否 | 分页偏移量，默认 0 |
| `Limit` | Int64 | 否 | 返回数量，默认 20 |

响应 Data：`Items`（CollectionContentItem，结构同 9.3）、`Paging`。

### 9.6 用户数据通用错误码

10001 参数错误、20001 鉴权失败、30001 频率限制、30002 配额限制、90001 内部错误。

---

## 10. CLI 设计初步建议

### 10.1 命令分层

```text
zhihu auth login --secret <ACCESS_SECRET>     # 保存 secret 并自动生成时间戳
zhihu search zhihu <QUERY> [--count N]
zhihu search global <QUERY> [--count N] [--filter ...] [--db all|realtime|static]
zhihu ask <QUERY> [--model fast|thinking|agent] [--stream]
zhihu hot [--limit N]
zhihu quota [--ids knowledge,tools]
zhihu kb list [--scope all|created|subscribed]
zhihu kb items <KB_ID> [--cursor STR] [--limit N]
zhihu kb upload <FILE> [--kb-id ID]
zhihu kb search <QUERY> [--kb-id ID]... [--scope personal|subscription|public]... [--limit N]
zhihu pdf upload <FILE>
zhihu pdf task <FILE_ID> [--idempotency-key KEY]
zhihu pdf status <TASK_ID>
zhihu pdf parse <FILE> [--timeout-secs N] [--idempotency-key KEY]   # 一键
zhihu ppt task <URL> --pages N [--idempotency-key KEY]
zhihu ppt status <TASK_ID>
zhihu ppt generate <URL> --pages N [--timeout-secs N] [--idempotency-key KEY]   # 一键
zhihu user contents [--type ...] [--offset N] [--limit N] [--sort-field ts|like_count] [--sort-order asc|desc] [--oauth-token T]
zhihu user followees [--offset N] [--limit N] [--oauth-token T]
zhihu user collections [--limit N] [--oauth-token T]
zhihu user favlists [--limit N] [--oauth-token T]
zhihu user favlist-contents <FAVLIST_URL_TOKEN> [--offset N] [--limit N] [--oauth-token T]
```

### 10.2 需要提前确认的设计点

1. **鉴权配置存储**：Access Secret 是写入本地配置文件（如 `~/.zhihu-cli/config.toml`），还是每次通过环境变量 `ZHIHU_ACCESS_SECRET` 传入？
2. **输出格式**：默认输出可读表格，还是 JSON？是否提供 `--json` / `--table` 开关？
3. **分页策略**：`global_search` 支持 `HasMore`，CLI 是否需要内置分页（`--page` / `--offset`）？当前文档未提供 offset 参数，需要实测或追问。
4. **流式输出**：`zhida` 流式响应如何与终端交互（是否逐字打印、是否支持 `--no-stream` 聚合）？
5. **Skill 包支持**：CLI 是否只需调用 HTTP API，还是也要能下载 / 执行本地 Skill zip？

### 10.3 实现注意事项

- 每个请求必须带 `X-Request-Timestamp`，建议用当前 Unix 秒级时间戳自动生成。
- `global_search` 的 `Filter` 需要 URL 编码，CLI 应提供参数封装避免用户手写表达式。
- `zhida` 兼容 OpenAI-style 接口语义，便于复用现有的 chat completion 客户端。
- 错误处理应区分：参数错误（10001）、鉴权失败（20001）、频率限制（30001）。
