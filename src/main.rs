mod cli;
mod detector;
mod error;
mod extractor;

use crate::error::Result;
use clap::Parser;
use std::fs;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    if !fs::exists(&cli.output)? {
        return Err("file does not exist".into());
    }

    let lang = detector::detect(&cli.output)?;
    println!("language is {}", lang);

    Ok(())
}
