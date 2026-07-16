mod cli;
mod cmd;
mod core;
mod output;

use crate::core::types::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Inject(args) => cmd::inject::run(args),
        cli::Commands::List(args) => cmd::list::run(args),
    }
}
