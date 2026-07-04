mod cli;
mod detector;
use anyhow::anyhow;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    let lang = detector::detect(&cli.output)
        .ok_or_else(|| anyhow!("unsupported file type: {:?}", cli.output))?;
    println!("language is {}", lang);

    Ok(())
}
