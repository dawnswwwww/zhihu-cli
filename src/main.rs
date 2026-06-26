mod cli;
mod client;
mod config;
mod error;
mod output;

use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    println!("{:?}", cli.command);
}
