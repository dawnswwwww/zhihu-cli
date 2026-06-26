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
