mod cli;
mod cmd;
mod core;

use crate::core::types::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    cmd::inject::run(cli.input, cli.output, cli.dry_run, cli.id)
}
