mod cli;
mod client;
mod commands;
mod config;
mod error;
mod output;

use clap::Parser;
use cli::{Cli, Command};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Auth { subcommand } => commands::auth::run(subcommand).await,
        Command::Search { subcommand } => commands::search::run(subcommand).await,
        Command::Ask(_) => {}
    }
}
