# 知乎开放平台 API 参考（CLI 设计用）

> 来源：https://developer.zhihu.com/docs
> 抓取时间：2026-06-26（第 1-4 节）；2026-09-07（第 5-9 节）；2026-09-24（第 6 节更新，新增第 10-11 节）
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

知识库文件上传/列表/内容列表/检索共用 `knowledge` 额度池；PDF 解析与 PPT 生成共用 `tools` 额度池；问题推荐与创作能力接口共用 `creator` 额度池；问题回答使用独立的 `question_answers` 额度池。

#### 请求参数

| Query 参数 | 类型 | 必填 | 说明 |
|------------|------|------|------|
| `APIIDs` | String | 否 | 逗号分隔的 API ID；参数只能出现一次；不传时返回当前已配置的可展示额度 |

可查询额度项（不传 `APIIDs` 时按此顺序返回；尚未接入额度配置的 API 会被省略，不会导致整个列表失败）：

| API ID | 名称 | 覆盖范围 |
|--------|------|----------|
| `global_search` | 全网搜 | 全网搜索 |
| `zhihu_search` | 知乎搜索 | 知乎内容搜索 |
| `hot_list` | 热榜 | 知乎热榜 |
| `question_answers` | 知乎问题回答 | 获取问题下的回答摘要 |
| `zhida_openai` | 直答 | 直答服务 |
| `tools` | 小工具 | PDF 解析及 PPT 生成 |
| `knowledge` | 知识库 | 知识库文件上传、知识库列表、知识库内容列表及知识库检索 |
| `user_data` | 知乎用户数据 | 用户创作列表、关注、收藏及收藏夹数据 |
| `creator` | 创作能力 | 个性化问题推荐、根据主题推荐问题、本人全文、评论、账号统计、单篇统计 |

账号关联多个租户时，查询结果会汇总相关租户额度。

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

## 10. 问题发现

### 10.1 API：`question_recommendations`（问题推荐）

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/user/question_recommendations` |
| 说明 | 根据当前 Access Secret 所属用户的画像，或用户指定的主题，推荐适合回答的知乎问题 |

| Query 参数 | 类型 | 必填 | 默认值 | 说明 |
|------------|------|------|--------|------|
| `Query` | String | 否 | — | 主题关键词。未提供时按当前用户画像推荐；提供时按主题推荐，去除首尾空白后不能为空 |
| `Count` | Int32 | 否 | 5 | 返回数量，范围 1-20 |

不传 `Query` 与传空字符串含义不同：`?Count=5` 使用画像推荐，`?Query=人工智能&Count=5` 使用主题推荐，`?Query=` 或纯空白返回 10001。两种模式均使用当前账号身份。

响应 Data：`Items`（数组），元素：`Title`（String，问题标题）、`Url`（String，问题链接）。返回条目可能不足 `Count` 或为空，不支持分页。

示例：

```json
{ "Code": 0, "Message": "success", "Data": { "Items": [ { "Title": "如何理解 AI Agent？", "Url": "https://www.zhihu.com/question/123" } ] } }
```

额度：与我的创作全文、评论、账号统计、单篇统计共用"创作能力"（`creator`）额度池，默认每个租户每个自然日 100 次，未实名等低额度用户为 10 次；实际额度以额度查询结果为准。日额度耗尽返回 30001。

错误码：10001 参数错误、20001 鉴权或授权失败、30001 调用频率/并发限制或当日额度超过限制、30002 额外配置的累计成功次数额度耗尽、30003 请求被风控拒绝、90001 服务内部错误。

### 10.2 API：`question_answers`（问题回答）

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/content/question_answers` |
| 说明 | 获取一个知乎问题下的回答列表。`Summary` 是服务返回的内容摘要或截取文本，不额外生成 AI 摘要，也不代表回答全文 |

| Query 参数 | 类型 | 必填 | 默认值 | 说明 |
|------------|------|------|--------|------|
| `QuestionUrl` | String | 是 | — | 完整的知乎问题 URL |
| `Offset` | Int64 | 否 | 0 | 分页偏移，不能为负数 |
| `Limit` | Int64 | 否 | 20 | 返回数量，范围 1-50 |

响应 Data：`Items`（数组）、`Paging`。Item：`ContentType`（固定 `answer`）、`ContentToken`（String，回答 Token）、`Url`、`Summary`。Paging：`IsEnd`（Bool）、`NextOffset`（Int64，`IsEnd=true` 时可省略）、`Totals`（Int64，未提供时可能不返回）。

```json
{ "Code": 0, "Message": "success", "Data": { "Items": [ { "ContentType": "answer", "ContentToken": "456", "Url": "https://www.zhihu.com/question/123/answer/456", "Summary": "这是一段回答摘要……" } ], "Paging": { "IsEnd": true, "Totals": 1 } } }
```

分页说明：无效或无摘要的回答会被过滤，单页 `Items` 数量可能少于 `Limit` 甚至为空。请使用服务返回的分页字段，不按过滤后的条数重新计算：

- `Paging.IsEnd=false` 时，按需将 `Paging.NextOffset` 作为下一次请求的 `Offset`。
- `Paging.IsEnd=true` 时停止翻页。请勿根据 `Items` 数量判断结束或自行计算偏移。
- 若 `IsEnd=false` 但缺少 `NextOffset`，停止自动翻页并报告分页信息不完整，避免重复请求。
- `Paging.Totals` 可能包含被过滤的项，不保证等于最终可读取的摘要数。

额度：使用独立的"知乎问题回答"（`question_answers`）额度池，默认每个租户每个自然日 100 次，未实名等低额度用户为 10 次。

错误码：10001 问题 URL 或分页参数错误，或者问题不存在、20001、30001、30002、30003、90001（含义同 10.1）。

---

## 11. 创作能力

通用约定：

- 仅支持当前 Access Secret 所属账号的本人数据；**不接受 OAuth 身份切换**（不使用 `X-OAuth-Token`），服务端从凭证解析本人身份，不能通过参数指定他人。
- 仅支持本人**已发布**的内容；草稿、未发布或其他不可用状态的内容不在支持范围内。
- 本节接口与两种问题推荐共用"创作能力"（`creator`）额度池，默认每个租户每个自然日共计 100 次，未实名等低额度用户为 10 次。
- 通用错误码：0 成功、10001 参数错误或内容不可用、20001 鉴权或授权失败、30001 调用频率/并发或当日额度超限、30002 额外配置的成功次数额度耗尽、30003 请求被风控拒绝、90001 服务内部错误。

### 11.1 API：`user_content_detail`（我的创作全文）

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/user/content_detail` |
| 说明 | 获取当前用户自己创作内容的全文。支持回答、文章、想法和视频 |

| Query 参数 | 类型 | 必填 | 说明 |
|------------|------|------|------|
| `ContentUrl` | String | 是 | 当前用户创作的回答、文章、想法或视频链接 |

响应 Data：`ContentType`（`answer`/`article`/`pin`/`zvideo`）、`ContentToken`（String，保留字符串精度）、`Url`、`Title`（可能为空字符串）、`Body`（全文，可能包含 HTML；展示时转义或安全清洗）。

- 视频类型仅返回关联正文，不提供视频文件下载。没有可用正文时返回内容不可用，不把空 `Body` 宣称为全文。
- 支持知乎 HTTPS 链接：`/answer/{id}`、`/question/{id}/answer/{id}`、`/p/{id}`（文章）、`/pin/{id}`、`/zvideo/{id}`。
- 链接无效、内容不存在或作者不属于当前用户时返回参数错误（10001），响应不会泄露作者归属信息。内容归属必须通过服务端验证，无法确认归属时不返回正文。全文与评论分别调用。

```bash
curl -G 'https://developer.zhihu.com/api/v1/user/content_detail' \
  --data-urlencode 'ContentUrl=https://www.zhihu.com/question/1/answer/2' \
  -H 'Authorization: Bearer <your_access_secret>' \
  -H "X-Request-Timestamp: $(date +%s)"
```

### 11.2 API：`user_content_comments`（我的创作评论）

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/user/content_comments` |
| 说明 | 分页获取当前用户自己创作内容下的评论，返回根评论及其子评论。支持回答、文章、想法和视频 |

| Query 参数 | 类型 | 必填 | 默认值 | 说明 |
|------------|------|------|--------|------|
| `ContentUrl` | String | 是 | — | 当前用户创作的内容链接 |
| `Offset` | Int64 | 否 | 0 | 分页偏移，不能为负数 |
| `Limit` | Int64 | 否 | 20 | 根评论返回数量，范围 1-50 |
| `Order` | String | 否 | `score` | `score` 按热度排序、`reverse` 按时间倒序、`ascending` 按时间正序 |

响应 Data：`Items`（数组）、`Paging`。`Items[]` 每项含 `Comment`（根评论）及 `Children[]`（子评论），两个层级使用相同字段：

| 字段 | 类型 | 说明 |
|------|------|------|
| `ID` | Int64 | 评论 ID；客户端应保留大整数精度 |
| `Type` | String | 评论类型 |
| `ReplyID` | Int64，可选 | 直接回复的评论 ID，值为 0 时省略 |
| `RootID` | Int64，可选 | 所属根评论 ID，值为 0 时省略 |
| `CreatedAt` | Int64 | 评论创建时间，Unix 秒级时间戳 |
| `Content` | String | 评论文本，按不可信内容处理 |
| `LikeCount` | Int64 | 点赞数 |
| `DislikeCount` | Int64 | 点踩数 |
| `AuthorToken` | String | 评论作者标识 |

`Children` 仅为上游附带的子评论，不保证完整。评论作者可能为他人，目标创作内容必须为当前账号本人所有。

分页：`Data.Paging.IsEnd` 为 Bool；`Totals` 和 `NextOffset` 为可选 Int64。`IsEnd=false` 时使用 `NextOffset` 作为下次 `Offset`；不要按 `Items` 条数累加，也不要因短页或空页停止。未提供或不递增的 `NextOffset` 应报告分页异常并停止自动翻页。`Totals` 沿用上游计数，不保证等于可遍历的评论数量。改变目标或排序应从 `Offset=0` 开始。

```json
{ "Code": 0, "Message": "success", "Data": { "Items": [ { "Comment": { "ID": 456, "Type": "article", "CreatedAt": 1742822400, "Content": "<p>示例评论</p>", "LikeCount": 2, "DislikeCount": 0, "AuthorToken": "example-user" }, "Children": [] } ], "Paging": { "IsEnd": true } } }
```

### 11.3 API：`creator_account_stats`（账号创作数据）

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/user/creator_account_stats` |
| 说明 | 获取当前 Access Secret 所属创作者的账号维度数据，包括内容指标、创作数量、粉丝概览和可用的受众画像 |

| Query 参数 | 类型 | 必填 | 说明 |
|------------|------|------|------|
| `ContentType` | String | 否 | `all`（默认）、`answer`、`article`、`pin` 或 `zvideo` |
| `StartDate` | String | 否 | 开始日期，格式 `YYYY-MM-DD`，需与 `EndDate` 同时提供 |
| `EndDate` | String | 否 | 结束日期，格式 `YYYY-MM-DD`，不得早于 `StartDate` |

日期必须成对提供或同时省略；省略时使用服务默认统计范围，不承诺固定天数。

响应 Data：`ContentType`（规范化后的内容类型）及以下可选字段（上游未提供的可选指标会省略）：

| Data 字段 | 类型 | 含义 |
|-----------|------|------|
| `Metrics` | Object，可选 | 内容指标，见下表 |
| `Audience` | Object，可选 | 受众画像及可选 `Status`/`Reason` |
| `CreationCounts` | Object，可选 | `Answer`、`Article`、`Video`、`Follower` 数量（Int64） |
| `Followers` | Object，可选 | 粉丝概览（FollowerSummary） |
| `FollowerDetails` | Object，可选 | `Daily` 日序列、`Today` 与 `Period` 周期数据 |
| `FollowerProfile` | Object，可选 | 画像及互动对象，需结合 `Status`/`Reason` 使用 |

字段阅读说明：JSON 字段名区分大小写。可选数值指标未返回时不应补零；已提供的数值指标为 0 时会保留。状态、字符串和集合字段也可能因零值、空字符串或空集合而省略，不能仅凭缺失判断上游未提供。Int64 计数字段需保留整数精度。比例沿用上游原值，单位未明确时不要自行乘 100 或拼接百分号。指标可用范围取决于内容类型和上游覆盖，不承诺所有字段同时存在。统计可能延迟，日期范围不保证对每项指标同时生效。

Metrics 字段（均可选）：`Updated`（String，统计更新时间）、`ViewCount`、`PlayCount`、`UpvoteCount`、`CommentCount`、`LikeCount`、`CollectCount`、`ShareCount`、`RepinCount`、`PublishCount`（均 Int64）、`ClickRate`、`ReadFinishedRate`、`PlayFinishedRate`（均 Float64）、`IncreasedUpvoteCount`、`DecreasedUpvoteCount`、`IncreasedLikeCount`、`DecreasedLikeCount`（Int64）、`Yesterday`/`Today`（Metrics，结构同本表）。

AudienceProfileItem：`Name`（String）、`Ratio`（Float64）、`Count`（Int64，缺失时不补零）。

AudienceContentItem：`ContentType`、`ContentToken`（String 保留精度）、`Title`、`FollowCount`（Int64）。

Audience：`Status`（String）、`Reason`（String）、`Source`/`Activeness`/`ActiveTime`/`Gender`/`Age`/`Interest`/`Location`/`OS`（均 Array\<AudienceProfileItem\>）、`Content`（Array\<AudienceContentItem\>）。

Followers（FollowerSummary）：`Total`（Int64，粉丝总数）、`Yesterday`（昨日净增）、`NewYesterday`（昨日新增）、`CancelledYesterday`（昨日取消关注）、`ActiveCount`（活跃粉丝数，Int64）、`ActiveRatio`（String，活跃粉丝占比）。

FollowerDaily：`Date`（String）、`NetIncrease`、`NewCount`、`UnfollowCount`、`HomepageVisitorCount`、`HomepageFollowCount`（Int64）、`HomepageConversionRate`（Float64）。

FollowerPeriod：`Date` 及 7/14/30 日三组指标：`NetIncrease{7,14,30}Days`、`NewCount{7,14,30}Days`、`UnfollowCount{7,14,30}Days`、`HomepageVisitorCount{7,14,30}Days`、`HomepageFollowCount{7,14,30}Days`（Int64）、`HomepageConversionRate{7,14,30}Days`（Float64）。

FollowerDetails：`Daily`（Array\<FollowerDaily\>）、`Today`（FollowerDaily，实际统计日期以 `Date` 为准）、`Period`（FollowerPeriod）。

FollowerCreatorItem：`Avatar`、`MemberToken`、`Name`（String）、`FollowCount`（Int64）。

FollowerInteractions：`Status`（Int32）、`Creators`（Array\<FollowerCreatorItem\>）、`Content`（Array\<AudienceContentItem\>）。

FollowerProfile：`Status`（Int32，值为 0 时省略，缺失不代表异常）、`Reason`（String）、`Audience`（Audience）、`Interactions`（FollowerInteractions）。

```json
{ "Code": 0, "Message": "success", "Data": { "ContentType": "all", "Metrics": { "Updated": "2026-09-08 12:00:00", "ViewCount": 100, "UpvoteCount": 10, "Yesterday": { "ViewCount": 20 } }, "Followers": { "Total": 50, "Yesterday": 2, "NewYesterday": 3, "CancelledYesterday": 1 } } }
```

### 11.4 API：`creator_content_stats`（单篇创作数据）

| 项目 | 值 |
|------|-----|
| URL | `GET https://developer.zhihu.com/api/v1/user/creator_content_stats` |
| 说明 | 获取当前用户自己创作的单篇内容数据，包括阅读、互动、转粉和可用的受众画像 |

| Query 参数 | 类型 | 必填 | 说明 |
|------------|------|------|------|
| `ContentUrl` | String | 是 | 当前用户创作的回答、文章、想法或视频链接 |
| `StartDate` | String | 否 | 开始日期，格式 `YYYY-MM-DD`，需与 `EndDate` 同时提供 |
| `EndDate` | String | 否 | 结束日期，格式 `YYYY-MM-DD`，不得早于 `StartDate` |

响应 Data：`Items`（数组，仅返回与已核验本人归属目标一致的数据；没有统计数据时可为空数组，不等同于各指标均为零）。Item：`ContentType`（`answer`/`article`/`pin`/`zvideo`）、`ContentToken`、`Url`、`Title`（空标题省略）、`Metrics`（可选）、`Audience`（可选）。

日期规则与字段阅读说明同 11.3。

Metrics 字段（均可选）：`Date`（String，统计日期）、`ViewCount`、`PlayCount`、`UpvoteCount`、`CommentCount`、`LikeCount`、`CollectCount`、`ShareCount`、`RepinCount`、`PublishCount`（Int64）、`ClickRate`、`ReadFinishedRate`、`PlayFinishedRate`（Float64）、`PageShowUV`（内容曝光用户数 UV）、`NewFollowerCount`（新增关注用户数）、`FollowerGain`（内容带来的转粉数量，Int64）、`FollowerConversionRate`（Float64，转粉率）、`PositiveInteractionRate`（String，正向互动率）、`IncreasedUpvoteCount`、`DecreasedUpvoteCount`、`IncreasedLikeCount`、`DecreasedLikeCount`、`UpvoteCount7Days`（Int64）、`Yesterday`/`Today`（Metrics，结构同本表）。

Audience 结构同 11.3（`Status`/`Reason` + 各分布数组 + `Content`）。

```json
{ "Code": 0, "Message": "success", "Data": { "Items": [ { "ContentType": "article", "ContentToken": "123", "Url": "https://zhuanlan.zhihu.com/p/123", "Title": "示例文章", "Metrics": { "Date": "2026-09-08", "ViewCount": 100, "UpvoteCount": 10 } } ] } }
```

---

## 12. CLI 设计初步建议

### 12.1 命令分层

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
zhihu question recommend [--query TOPIC] [--count N]
zhihu question answers <QUESTION_URL> [--offset N] [--limit N]
zhihu creator detail <CONTENT_URL>
zhihu creator comments <CONTENT_URL> [--offset N] [--limit N] [--order score|reverse|ascending]
zhihu creator account-stats [--type all|answer|article|pin|zvideo] [--start-date YYYY-MM-DD --end-date YYYY-MM-DD]
zhihu creator content-stats <CONTENT_URL> [--start-date YYYY-MM-DD --end-date YYYY-MM-DD]
```

### 12.2 需要提前确认的设计点

1. **鉴权配置存储**：Access Secret 是写入本地配置文件（如 `~/.zhihu-cli/config.toml`），还是每次通过环境变量 `ZHIHU_ACCESS_SECRET` 传入？
2. **输出格式**：默认输出可读表格，还是 JSON？是否提供 `--json` / `--table` 开关？
3. **分页策略**：`global_search` 支持 `HasMore`，CLI 是否需要内置分页（`--page` / `--offset`）？当前文档未提供 offset 参数，需要实测或追问。
4. **流式输出**：`zhida` 流式响应如何与终端交互（是否逐字打印、是否支持 `--no-stream` 聚合）？
5. **Skill 包支持**：CLI 是否只需调用 HTTP API，还是也要能下载 / 执行本地 Skill zip？

### 12.3 实现注意事项

- 每个请求必须带 `X-Request-Timestamp`，建议用当前 Unix 秒级时间戳自动生成。
- `global_search` 的 `Filter` 需要 URL 编码，CLI 应提供参数封装避免用户手写表达式。
- `zhida` 兼容 OpenAI-style 接口语义，便于复用现有的 chat completion 客户端。
- 错误处理应区分：参数错误（10001）、鉴权失败（20001）、频率限制（30001）。
