mod dev_server;
mod agent;
mod audit;
mod baseplates;
mod cli;
mod cmd;
mod shell;
mod config;
mod config_cli;
mod detect;
mod engines;
mod install;
mod pkgmgr;
mod project;
mod report;
mod score;
mod tools;
mod tui;
mod types;
mod upgrade;
mod workspace;

use clap::Parser;

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();
    std::process::exit(cli::run(cli).await);
}
