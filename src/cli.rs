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
}
