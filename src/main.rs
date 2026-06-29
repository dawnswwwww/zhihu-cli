use clap::Parser;
use zhihu_cli::cli::{Cli, Command};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Auth { subcommand } => zhihu_cli::commands::auth::run(subcommand).await,
        Command::Search { subcommand } => zhihu_cli::commands::search::run(subcommand).await,
        Command::Ask(args) => zhihu_cli::commands::ask::run(args).await,
        Command::Hot(args) => {
            if let Err(e) = zhihu_cli::commands::hot::run(args).await {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }
}
