mod cli;
mod cmd;
mod core;

use crate::core::types::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Inject(args) => {
            cmd::inject::run(args.input, args.output, args.dry_run, args.id)
        }
        cli::Commands::List(_) => Ok(()),
    }
}
