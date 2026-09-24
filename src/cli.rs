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
    /// Show Zhihu hot list
    Hot(HotArgs),
    /// Query daily free quota usage
    Quota(QuotaArgs),
    /// Zhihu user data commands (contents, followees, collections, favlists)
    User {
        #[command(subcommand)]
        subcommand: UserCommand,
    },
    /// Knowledge base commands
    Kb {
        #[command(subcommand)]
        subcommand: KbCommand,
    },
    /// PDF parse commands
    Pdf {
        #[command(subcommand)]
        subcommand: PdfCommand,
    },
    /// PPT generation commands
    Ppt {
        #[command(subcommand)]
        subcommand: PptCommand,
    },
    /// Zhihu question commands (recommendations, answers)
    Question {
        #[command(subcommand)]
        subcommand: QuestionCommand,
    },
    /// Creator data commands (detail, comments, account/content stats)
    Creator {
        #[command(subcommand)]
        subcommand: CreatorCommand,
    },
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

#[derive(Debug, clap::Args)]
pub struct HotArgs {
    /// Number of results to return
    #[arg(long, default_value = "30")]
    pub limit: i32,
}

#[derive(Debug, clap::Args)]
pub struct QuotaArgs {
    /// Comma-separated API IDs to filter (e.g. knowledge,zhihu_search); omit for all
    #[arg(long, value_delimiter = ',')]
    pub ids: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum UserCommand {
    /// List the user's created contents (answers, articles, videos, pins, questions)
    Contents {
        /// Content type filter
        #[arg(long = "type", value_enum, default_value = "all")]
        content_type: UserContentType,
        /// Pagination offset
        #[arg(long, default_value = "0")]
        offset: i64,
        /// Number of results (API max 50)
        #[arg(long, default_value = "20")]
        limit: i64,
        /// Sort field
        #[arg(long, value_enum, default_value = "ts")]
        sort_field: SortField,
        /// Sort order
        #[arg(long, value_enum, default_value = "desc")]
        sort_order: SortOrder,
        /// OAuth token: query that authorized user instead of yourself
        #[arg(long)]
        oauth_token: Option<String>,
    },
    /// List the users the user follows
    Followees {
        /// Pagination offset
        #[arg(long, default_value = "0")]
        offset: i64,
        /// Number of results (API max 50)
        #[arg(long, default_value = "20")]
        limit: i64,
        /// OAuth token: query that authorized user instead of yourself
        #[arg(long)]
        oauth_token: Option<String>,
    },
    /// List the user's recently collected contents
    Collections {
        /// Number of results
        #[arg(long, default_value = "20")]
        limit: i64,
        /// OAuth token: query that authorized user instead of yourself
        #[arg(long)]
        oauth_token: Option<String>,
    },
    /// List the user's favlists
    Favlists {
        /// Number of results
        #[arg(long, default_value = "20")]
        limit: i64,
        /// OAuth token: query that authorized user instead of yourself
        #[arg(long)]
        oauth_token: Option<String>,
    },
    /// List the contents of a favlist
    FavlistContents {
        /// Favlist URL token (from `zhihu user favlists` UrlToken field)
        favlist_url_token: i64,
        /// Pagination offset
        #[arg(long, default_value = "0")]
        offset: i64,
        /// Number of results
        #[arg(long, default_value = "20")]
        limit: i64,
        /// OAuth token: query that authorized user instead of yourself
        #[arg(long)]
        oauth_token: Option<String>,
    },
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

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum UserContentType {
    #[default]
    All,
    Answer,
    Article,
    Zvideo,
    Pin,
    Question,
}

impl UserContentType {
    pub fn api_name(&self) -> &'static str {
        match self {
            UserContentType::All => "all",
            UserContentType::Answer => "answer",
            UserContentType::Article => "article",
            UserContentType::Zvideo => "zvideo",
            UserContentType::Pin => "pin",
            UserContentType::Question => "question",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum SortField {
    #[default]
    Ts,
    #[value(name = "like_count")]
    LikeCount,
}

impl SortField {
    pub fn api_name(&self) -> &'static str {
        match self {
            SortField::Ts => "ts",
            SortField::LikeCount => "like_count",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum SortOrder {
    #[default]
    Desc,
    Asc,
}

impl SortOrder {
    pub fn api_name(&self) -> &'static str {
        match self {
            SortOrder::Desc => "desc",
            SortOrder::Asc => "asc",
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum QuestionCommand {
    /// Recommend questions (profile-based, or filtered by topic keywords)
    Recommend {
        /// Topic keywords; omit for profile-based recommendations
        #[arg(long)]
        query: Option<String>,
        /// Number of recommendations (API max 20)
        #[arg(long, default_value = "5")]
        count: i32,
    },
    /// List the answers to a question
    Answers {
        /// Full Zhihu question URL
        question_url: String,
        /// Pagination offset
        #[arg(long, default_value = "0")]
        offset: i64,
        /// Number of results (API max 50)
        #[arg(long, default_value = "20")]
        limit: i64,
    },
}

#[derive(Debug, Subcommand)]
pub enum CreatorCommand {
    /// Get the full content of one of your creations
    Detail {
        /// URL of your own answer/article/pin/zvideo
        content_url: String,
    },
    /// List the comments on one of your creations
    Comments {
        /// URL of your own answer/article/pin/zvideo
        content_url: String,
        /// Pagination offset
        #[arg(long, default_value = "0")]
        offset: i64,
        /// Number of results (API max 50)
        #[arg(long, default_value = "20")]
        limit: i64,
        /// Comment sort order
        #[arg(long, value_enum, default_value = "score")]
        order: CommentOrder,
    },
    /// Account-level creator stats
    AccountStats {
        /// Content type filter
        #[arg(long = "type", value_enum, default_value = "all")]
        content_type: CreatorContentType,
        /// Start date (YYYY-MM-DD); must be given together with --end-date
        #[arg(long)]
        start_date: Option<String>,
        /// End date (YYYY-MM-DD); must be given together with --start-date
        #[arg(long)]
        end_date: Option<String>,
    },
    /// Per-content creator stats
    ContentStats {
        /// URL of your own answer/article/pin/zvideo
        content_url: String,
        /// Start date (YYYY-MM-DD); must be given together with --end-date
        #[arg(long)]
        start_date: Option<String>,
        /// End date (YYYY-MM-DD); must be given together with --start-date
        #[arg(long)]
        end_date: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum CreatorContentType {
    #[default]
    All,
    Answer,
    Article,
    Pin,
    Zvideo,
}

impl CreatorContentType {
    pub fn api_name(&self) -> &'static str {
        match self {
            CreatorContentType::All => "all",
            CreatorContentType::Answer => "answer",
            CreatorContentType::Article => "article",
            CreatorContentType::Pin => "pin",
            CreatorContentType::Zvideo => "zvideo",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum CommentOrder {
    #[default]
    Score,
    Reverse,
    Ascending,
}

impl CommentOrder {
    pub fn api_name(&self) -> &'static str {
        match self {
            CommentOrder::Score => "score",
            CommentOrder::Reverse => "reverse",
            CommentOrder::Ascending => "ascending",
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum KbCommand {
    /// List knowledge bases you created or subscribed
    List {
        /// Filter by your relation to the knowledge base
        #[arg(long, value_enum, default_value = "all")]
        scope: KbScope,
    },
    /// List the contents of a knowledge base
    Items {
        /// Knowledge base ID (from `zhihu kb list`)
        kb_id: String,
        /// Page cursor from the previous response's NextCursor field
        #[arg(long)]
        cursor: Option<String>,
        /// Items per page (API max 20)
        #[arg(long, default_value = "20")]
        limit: i32,
    },
    /// Upload a file to a knowledge base (your default KB if omitted)
    Upload {
        /// Path to the file to upload
        file: String,
        /// Target knowledge base ID; omit for your default KB
        #[arg(long = "kb-id")]
        kb_id: Option<String>,
    },
    /// RAG search across knowledge bases and/or recall scopes
    Search {
        /// Search query
        query: String,
        /// Knowledge base ID (repeatable)
        #[arg(long = "kb-id")]
        kb_ids: Vec<String>,
        /// Recall scope: personal, subscription, or public (repeatable)
        #[arg(long = "scope", value_enum)]
        scopes: Vec<RecallScope>,
        /// Number of documents to return (API max 10)
        #[arg(long, default_value = "10")]
        limit: i32,
    },
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum KbScope {
    #[default]
    All,
    Created,
    Subscribed,
}

impl KbScope {
    pub fn api_name(&self) -> &'static str {
        match self {
            KbScope::All => "all",
            KbScope::Created => "created",
            KbScope::Subscribed => "subscribed",
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum RecallScope {
    #[value(name = "personal")]
    Personal,
    #[value(name = "subscription")]
    Subscription,
    #[value(name = "public")]
    Public,
}

impl RecallScope {
    pub fn api_name(&self) -> &'static str {
        match self {
            RecallScope::Personal => "personal",
            RecallScope::Subscription => "subscription",
            RecallScope::Public => "public",
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum PdfCommand {
    /// Upload a PDF file and get a file_id (valid for 24h)
    Upload {
        /// Path to the PDF file
        file: String,
    },
    /// Create a PDF parse task from a file_id
    Task {
        /// File resource ID from `zhihu pdf upload`
        file_id: String,
        /// Idempotency key: same key with same params returns the same task
        #[arg(long)]
        idempotency_key: Option<String>,
    },
    /// Query a PDF parse task status
    Status {
        /// Task ID from `zhihu pdf task`
        task_id: String,
    },
    /// One-shot: upload, create task, and wait for the parse result
    Parse {
        /// Path to the PDF file
        file: String,
        /// Max seconds to wait for the task to finish
        #[arg(long, default_value = "600")]
        timeout_secs: u64,
        /// Idempotency key: same key with same params returns the same task
        #[arg(long)]
        idempotency_key: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum PptCommand {
    /// Create a PPT generation task from a Zhihu answer/article URL
    Task {
        /// Zhihu answer or article URL
        resource_url: String,
        /// Desired page count (API range 6-21)
        #[arg(long)]
        pages: i32,
        /// Idempotency key: same key with same params returns the same task
        #[arg(long)]
        idempotency_key: Option<String>,
    },
    /// Query a PPT generation task status
    Status {
        /// Task ID from `zhihu ppt task`
        task_id: String,
    },
    /// One-shot: create a task and wait for the PPTX download link
    Generate {
        /// Zhihu answer or article URL
        resource_url: String,
        /// Desired page count (API range 6-21)
        #[arg(long)]
        pages: i32,
        /// Max seconds to wait for the task to finish
        #[arg(long, default_value = "600")]
        timeout_secs: u64,
        /// Idempotency key: same key with same params returns the same task
        #[arg(long)]
        idempotency_key: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ask_defaults_to_thinking() {
        let cli = Cli::parse_from(["zhihu", "ask", "hello"]);
        match cli.command {
            Command::Ask(args) => {
                assert_eq!(args.query, "hello");
                assert!(matches!(args.model, ModelTier::Thinking));
                assert!(!args.stream);
            }
            _ => panic!("expected Ask command"),
        }
    }

    #[test]
    fn parse_ask_with_stream_and_model() {
        let cli = Cli::parse_from(["zhihu", "ask", "hello", "--stream", "--model", "agent"]);
        match cli.command {
            Command::Ask(args) => {
                assert!(args.stream);
                assert!(matches!(args.model, ModelTier::Agent));
            }
            _ => panic!("expected Ask command"),
        }
    }

    #[test]
    fn parse_search_zhihu_with_count() {
        let cli = Cli::parse_from(["zhihu", "search", "zhihu", "query", "--count", "5"]);
        match cli.command {
            Command::Search {
                subcommand: SearchCommand::Zhihu { query, count },
            } => {
                assert_eq!(query, "query");
                assert_eq!(count, 5);
            }
            _ => panic!("expected search zhihu"),
        }
    }

    #[test]
    fn parse_search_global_with_filter_and_db() {
        let cli = Cli::parse_from([
            "zhihu",
            "search",
            "global",
            "query",
            "--count",
            "15",
            "--filter",
            "host==\"example.com\"",
            "--db",
            "realtime",
        ]);
        match cli.command {
            Command::Search {
                subcommand:
                    SearchCommand::Global {
                        query,
                        count,
                        filter,
                        db,
                    },
            } => {
                assert_eq!(query, "query");
                assert_eq!(count, 15);
                assert_eq!(filter, Some("host==\"example.com\"".into()));
                assert!(matches!(db, SearchDb::Realtime));
            }
            _ => panic!("expected search global"),
        }
    }

    #[test]
    fn model_tier_api_names() {
        assert_eq!(ModelTier::Fast.api_name(), "zhida-fast-1p5");
        assert_eq!(ModelTier::Thinking.api_name(), "zhida-thinking-1p5");
        assert_eq!(ModelTier::Agent.api_name(), "zhida-agent");
    }

    #[test]
    fn search_db_api_names() {
        assert_eq!(SearchDb::All.api_name(), "all");
        assert_eq!(SearchDb::Realtime.api_name(), "realtime");
        assert_eq!(SearchDb::Static.api_name(), "static");
    }

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

    #[test]
    fn parse_quota_defaults_to_no_ids() {
        let cli = Cli::parse_from(["zhihu", "quota"]);
        match cli.command {
            Command::Quota(args) => {
                assert!(args.ids.is_empty());
            }
            _ => panic!("expected Quota command"),
        }
    }

    #[test]
    fn parse_quota_splits_comma_separated_ids() {
        let cli = Cli::parse_from(["zhihu", "quota", "--ids", "knowledge,zhihu_search"]);
        match cli.command {
            Command::Quota(args) => {
                assert_eq!(args.ids, vec!["knowledge", "zhihu_search"]);
            }
            _ => panic!("expected Quota command"),
        }
    }

    #[test]
    fn parse_user_contents_defaults() {
        let cli = Cli::parse_from(["zhihu", "user", "contents"]);
        match cli.command {
            Command::User { subcommand: UserCommand::Contents { content_type, offset, limit, sort_field, sort_order, oauth_token } } => {
                assert!(matches!(content_type, UserContentType::All));
                assert_eq!(offset, 0);
                assert_eq!(limit, 20);
                assert!(matches!(sort_field, SortField::Ts));
                assert!(matches!(sort_order, SortOrder::Desc));
                assert!(oauth_token.is_none());
            }
            _ => panic!("expected user contents"),
        }
    }

    #[test]
    fn parse_user_contents_with_all_options() {
        let cli = Cli::parse_from([
            "zhihu", "user", "contents",
            "--type", "article",
            "--offset", "40",
            "--limit", "50",
            "--sort-field", "like_count",
            "--sort-order", "asc",
            "--oauth-token", "tok-1",
        ]);
        match cli.command {
            Command::User { subcommand: UserCommand::Contents { content_type, offset, limit, sort_field, sort_order, oauth_token } } => {
                assert!(matches!(content_type, UserContentType::Article));
                assert_eq!(offset, 40);
                assert_eq!(limit, 50);
                assert!(matches!(sort_field, SortField::LikeCount));
                assert!(matches!(sort_order, SortOrder::Asc));
                assert_eq!(oauth_token.as_deref(), Some("tok-1"));
            }
            _ => panic!("expected user contents"),
        }
    }

    #[test]
    fn parse_user_followees_defaults() {
        let cli = Cli::parse_from(["zhihu", "user", "followees"]);
        match cli.command {
            Command::User { subcommand: UserCommand::Followees { offset, limit, oauth_token } } => {
                assert_eq!(offset, 0);
                assert_eq!(limit, 20);
                assert!(oauth_token.is_none());
            }
            _ => panic!("expected user followees"),
        }
    }

    #[test]
    fn parse_user_collections_defaults() {
        let cli = Cli::parse_from(["zhihu", "user", "collections"]);
        match cli.command {
            Command::User { subcommand: UserCommand::Collections { limit, oauth_token } } => {
                assert_eq!(limit, 20);
                assert!(oauth_token.is_none());
            }
            _ => panic!("expected user collections"),
        }
    }

    #[test]
    fn parse_user_favlists_with_limit() {
        let cli = Cli::parse_from(["zhihu", "user", "favlists", "--limit", "5"]);
        match cli.command {
            Command::User { subcommand: UserCommand::Favlists { limit, oauth_token } } => {
                assert_eq!(limit, 5);
                assert!(oauth_token.is_none());
            }
            _ => panic!("expected user favlists"),
        }
    }

    #[test]
    fn parse_user_favlist_contents_with_token() {
        let cli = Cli::parse_from([
            "zhihu", "user", "favlist-contents", "123456789",
            "--offset", "10", "--limit", "15",
        ]);
        match cli.command {
            Command::User { subcommand: UserCommand::FavlistContents { favlist_url_token, offset, limit, oauth_token } } => {
                assert_eq!(favlist_url_token, 123456789);
                assert_eq!(offset, 10);
                assert_eq!(limit, 15);
                assert!(oauth_token.is_none());
            }
            _ => panic!("expected user favlist-contents"),
        }
    }

    #[test]
    fn parse_kb_list_defaults_to_all() {
        let cli = Cli::parse_from(["zhihu", "kb", "list"]);
        match cli.command {
            Command::Kb { subcommand: KbCommand::List { scope } } => {
                assert!(matches!(scope, KbScope::All));
            }
            _ => panic!("expected kb list"),
        }
    }

    #[test]
    fn parse_kb_list_with_scope() {
        let cli = Cli::parse_from(["zhihu", "kb", "list", "--scope", "subscribed"]);
        match cli.command {
            Command::Kb { subcommand: KbCommand::List { scope } } => {
                assert!(matches!(scope, KbScope::Subscribed));
            }
            _ => panic!("expected kb list"),
        }
    }

    #[test]
    fn parse_kb_items_with_cursor_and_limit() {
        let cli = Cli::parse_from(["zhihu", "kb", "items", "kb-1", "--cursor", "c-2", "--limit", "5"]);
        match cli.command {
            Command::Kb { subcommand: KbCommand::Items { kb_id, cursor, limit } } => {
                assert_eq!(kb_id, "kb-1");
                assert_eq!(cursor.as_deref(), Some("c-2"));
                assert_eq!(limit, 5);
            }
            _ => panic!("expected kb items"),
        }
    }

    #[test]
    fn parse_kb_upload_with_kb_id() {
        let cli = Cli::parse_from(["zhihu", "kb", "upload", "./doc.pdf", "--kb-id", "kb-1"]);
        match cli.command {
            Command::Kb { subcommand: KbCommand::Upload { file, kb_id } } => {
                assert_eq!(file, "./doc.pdf");
                assert_eq!(kb_id.as_deref(), Some("kb-1"));
            }
            _ => panic!("expected kb upload"),
        }
    }

    #[test]
    fn parse_kb_search_with_repeatable_ids_and_scopes() {
        let cli = Cli::parse_from([
            "zhihu", "kb", "search", "退款规则",
            "--kb-id", "kb-1", "--kb-id", "kb-2",
            "--scope", "personal", "--scope", "public",
            "--limit", "5",
        ]);
        match cli.command {
            Command::Kb { subcommand: KbCommand::Search { query, kb_ids, scopes, limit } } => {
                assert_eq!(query, "退款规则");
                assert_eq!(kb_ids, vec!["kb-1", "kb-2"]);
                assert!(scopes.iter().any(|s| matches!(s, RecallScope::Personal)));
                assert!(scopes.iter().any(|s| matches!(s, RecallScope::Public)));
                assert_eq!(limit, 5);
            }
            _ => panic!("expected kb search"),
        }
    }

    #[test]
    fn parse_pdf_upload_takes_file() {
        let cli = Cli::parse_from(["zhihu", "pdf", "upload", "./report.pdf"]);
        match cli.command {
            Command::Pdf { subcommand: PdfCommand::Upload { file } } => {
                assert_eq!(file, "./report.pdf");
            }
            _ => panic!("expected pdf upload"),
        }
    }

    #[test]
    fn parse_pdf_task_with_idempotency_key() {
        let cli = Cli::parse_from(["zhihu", "pdf", "task", "file_abc", "--idempotency-key", "k1"]);
        match cli.command {
            Command::Pdf { subcommand: PdfCommand::Task { file_id, idempotency_key } } => {
                assert_eq!(file_id, "file_abc");
                assert_eq!(idempotency_key.as_deref(), Some("k1"));
            }
            _ => panic!("expected pdf task"),
        }
    }

    #[test]
    fn parse_pdf_status_takes_task_id() {
        let cli = Cli::parse_from(["zhihu", "pdf", "status", "pdf_1"]);
        match cli.command {
            Command::Pdf { subcommand: PdfCommand::Status { task_id } } => {
                assert_eq!(task_id, "pdf_1");
            }
            _ => panic!("expected pdf status"),
        }
    }

    #[test]
    fn parse_pdf_parse_defaults_timeout_and_key() {
        let cli = Cli::parse_from(["zhihu", "pdf", "parse", "./report.pdf"]);
        match cli.command {
            Command::Pdf { subcommand: PdfCommand::Parse { file, timeout_secs, idempotency_key } } => {
                assert_eq!(file, "./report.pdf");
                assert_eq!(timeout_secs, 600);
                assert!(idempotency_key.is_none());
            }
            _ => panic!("expected pdf parse"),
        }
    }

    #[test]
    fn parse_pdf_parse_with_timeout() {
        let cli = Cli::parse_from(["zhihu", "pdf", "parse", "./r.pdf", "--timeout-secs", "30"]);
        match cli.command {
            Command::Pdf { subcommand: PdfCommand::Parse { timeout_secs, .. } } => {
                assert_eq!(timeout_secs, 30);
            }
            _ => panic!("expected pdf parse"),
        }
    }

    #[test]
    fn parse_ppt_task_requires_pages() {
        let cli = Cli::parse_from(["zhihu", "ppt", "task", "https://zhuanlan.zhihu.com/p/1", "--pages", "12"]);
        match cli.command {
            Command::Ppt { subcommand: PptCommand::Task { resource_url, pages, idempotency_key } } => {
                assert_eq!(resource_url, "https://zhuanlan.zhihu.com/p/1");
                assert_eq!(pages, 12);
                assert!(idempotency_key.is_none());
            }
            _ => panic!("expected ppt task"),
        }
    }

    #[test]
    fn parse_ppt_status_takes_task_id() {
        let cli = Cli::parse_from(["zhihu", "ppt", "status", "ppt_1"]);
        match cli.command {
            Command::Ppt { subcommand: PptCommand::Status { task_id } } => {
                assert_eq!(task_id, "ppt_1");
            }
            _ => panic!("expected ppt status"),
        }
    }

    #[test]
    fn parse_ppt_generate_defaults_timeout() {
        let cli = Cli::parse_from([
            "zhihu", "ppt", "generate", "https://zhuanlan.zhihu.com/p/1",
            "--pages", "8", "--idempotency-key", "k1",
        ]);
        match cli.command {
            Command::Ppt { subcommand: PptCommand::Generate { resource_url, pages, timeout_secs, idempotency_key } } => {
                assert_eq!(resource_url, "https://zhuanlan.zhihu.com/p/1");
                assert_eq!(pages, 8);
                assert_eq!(timeout_secs, 600);
                assert_eq!(idempotency_key.as_deref(), Some("k1"));
            }
            _ => panic!("expected ppt generate"),
        }
    }

    #[test]
    fn parse_question_recommend_defaults() {
        let cli = Cli::parse_from(["zhihu", "question", "recommend"]);
        match cli.command {
            Command::Question { subcommand: QuestionCommand::Recommend { query, count } } => {
                assert!(query.is_none());
                assert_eq!(count, 5);
            }
            _ => panic!("expected question recommend"),
        }
    }

    #[test]
    fn parse_question_recommend_with_query_and_count() {
        let cli = Cli::parse_from([
            "zhihu", "question", "recommend", "--query", "rust", "--count", "10",
        ]);
        match cli.command {
            Command::Question { subcommand: QuestionCommand::Recommend { query, count } } => {
                assert_eq!(query.as_deref(), Some("rust"));
                assert_eq!(count, 10);
            }
            _ => panic!("expected question recommend"),
        }
    }

    #[test]
    fn parse_question_answers_defaults() {
        let cli = Cli::parse_from(["zhihu", "question", "answers", "https://www.zhihu.com/question/1"]);
        match cli.command {
            Command::Question { subcommand: QuestionCommand::Answers { question_url, offset, limit } } => {
                assert_eq!(question_url, "https://www.zhihu.com/question/1");
                assert_eq!(offset, 0);
                assert_eq!(limit, 20);
            }
            _ => panic!("expected question answers"),
        }
    }

    #[test]
    fn parse_question_answers_with_offset_and_limit() {
        let cli = Cli::parse_from([
            "zhihu", "question", "answers", "https://www.zhihu.com/question/1",
            "--offset", "40", "--limit", "50",
        ]);
        match cli.command {
            Command::Question { subcommand: QuestionCommand::Answers { question_url, offset, limit } } => {
                assert_eq!(question_url, "https://www.zhihu.com/question/1");
                assert_eq!(offset, 40);
                assert_eq!(limit, 50);
            }
            _ => panic!("expected question answers"),
        }
    }

    #[test]
    fn parse_creator_detail_takes_content_url() {
        let cli = Cli::parse_from(["zhihu", "creator", "detail", "https://www.zhihu.com/answer/1"]);
        match cli.command {
            Command::Creator { subcommand: CreatorCommand::Detail { content_url } } => {
                assert_eq!(content_url, "https://www.zhihu.com/answer/1");
            }
            _ => panic!("expected creator detail"),
        }
    }

    #[test]
    fn parse_creator_comments_defaults() {
        let cli = Cli::parse_from(["zhihu", "creator", "comments", "https://www.zhihu.com/answer/1"]);
        match cli.command {
            Command::Creator { subcommand: CreatorCommand::Comments { content_url, offset, limit, order } } => {
                assert_eq!(content_url, "https://www.zhihu.com/answer/1");
                assert_eq!(offset, 0);
                assert_eq!(limit, 20);
                assert!(matches!(order, CommentOrder::Score));
            }
            _ => panic!("expected creator comments"),
        }
    }

    #[test]
    fn parse_creator_comments_with_all_options() {
        let cli = Cli::parse_from([
            "zhihu", "creator", "comments", "https://www.zhihu.com/answer/1",
            "--offset", "10", "--limit", "30", "--order", "ascending",
        ]);
        match cli.command {
            Command::Creator { subcommand: CreatorCommand::Comments { content_url, offset, limit, order } } => {
                assert_eq!(content_url, "https://www.zhihu.com/answer/1");
                assert_eq!(offset, 10);
                assert_eq!(limit, 30);
                assert!(matches!(order, CommentOrder::Ascending));
            }
            _ => panic!("expected creator comments"),
        }
    }

    #[test]
    fn parse_creator_account_stats_defaults() {
        let cli = Cli::parse_from(["zhihu", "creator", "account-stats"]);
        match cli.command {
            Command::Creator { subcommand: CreatorCommand::AccountStats { content_type, start_date, end_date } } => {
                assert!(matches!(content_type, CreatorContentType::All));
                assert!(start_date.is_none());
                assert!(end_date.is_none());
            }
            _ => panic!("expected creator account-stats"),
        }
    }

    #[test]
    fn parse_creator_account_stats_with_type_and_dates() {
        let cli = Cli::parse_from([
            "zhihu", "creator", "account-stats",
            "--type", "zvideo", "--start-date", "2026-01-01", "--end-date", "2026-01-31",
        ]);
        match cli.command {
            Command::Creator { subcommand: CreatorCommand::AccountStats { content_type, start_date, end_date } } => {
                assert!(matches!(content_type, CreatorContentType::Zvideo));
                assert_eq!(start_date.as_deref(), Some("2026-01-01"));
                assert_eq!(end_date.as_deref(), Some("2026-01-31"));
            }
            _ => panic!("expected creator account-stats"),
        }
    }

    #[test]
    fn parse_creator_content_stats_with_dates() {
        let cli = Cli::parse_from([
            "zhihu", "creator", "content-stats", "https://zhuanlan.zhihu.com/p/1",
            "--start-date", "2026-02-01", "--end-date", "2026-02-28",
        ]);
        match cli.command {
            Command::Creator { subcommand: CreatorCommand::ContentStats { content_url, start_date, end_date } } => {
                assert_eq!(content_url, "https://zhuanlan.zhihu.com/p/1");
                assert_eq!(start_date.as_deref(), Some("2026-02-01"));
                assert_eq!(end_date.as_deref(), Some("2026-02-28"));
            }
            _ => panic!("expected creator content-stats"),
        }
    }

    #[test]
    fn creator_content_type_api_names() {
        assert_eq!(CreatorContentType::All.api_name(), "all");
        assert_eq!(CreatorContentType::Answer.api_name(), "answer");
        assert_eq!(CreatorContentType::Article.api_name(), "article");
        assert_eq!(CreatorContentType::Pin.api_name(), "pin");
        assert_eq!(CreatorContentType::Zvideo.api_name(), "zvideo");
    }

    #[test]
    fn comment_order_api_names() {
        assert_eq!(CommentOrder::Score.api_name(), "score");
        assert_eq!(CommentOrder::Reverse.api_name(), "reverse");
        assert_eq!(CommentOrder::Ascending.api_name(), "ascending");
    }
}
