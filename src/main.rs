mod cli;
mod cmd;
mod injector;
mod output;
mod parser;
mod types;
mod validator;

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Inject(args) => cmd::inject::run(args),
        cli::Commands::List(args) => cmd::list::run(args),
    }
}
