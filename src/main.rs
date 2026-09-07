use clap::Parser;
use zhihu_cli::cli::{Cli, Command};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Auth { subcommand } => zhihu_cli::commands::auth::run(subcommand).await,
        Command::Search { subcommand } => zhihu_cli::commands::search::run(subcommand).await,
        Command::Ask(args) => zhihu_cli::commands::ask::run(args).await,
        Command::Hot(args) => zhihu_cli::commands::hot::run(args).await,
        Command::Quota(args) => zhihu_cli::commands::quota::run(args).await,
        Command::User { subcommand } => zhihu_cli::commands::user::run(subcommand).await,
        Command::Kb { subcommand } => zhihu_cli::commands::knowledge::run(subcommand).await,
        Command::Pdf { subcommand } => zhihu_cli::commands::pdf::run(subcommand).await,
        Command::Ppt { subcommand } => zhihu_cli::commands::ppt::run(subcommand).await,
    }
}
