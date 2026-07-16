mod checker;
mod cli;
mod cmd;
mod injector;
mod output;
mod parser;
mod types;

use crate::types::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Inject(args) => cmd::inject::run(args),
        cli::Commands::List(args) => cmd::list::run(args),
    }
}
