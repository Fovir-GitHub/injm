mod cli;
mod detector;
mod error;
mod extractor;
mod injector;
mod io;
mod marker;

use crate::error::Result;
use clap::Parser;
use std::fs;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    if !fs::exists(&cli.output)? {
        return Err("file does not exist".into());
    }

    let lang = detector::detect(&cli.output)?;
    let content = fs::read_to_string(&cli.output)?;
    let comments = extractor::extract_comments(&content, lang)?;
    let blocks = marker::extract_marker_blocks(&comments)?;
    let stdin = io::read_stdin()?;
    let replaced = injector::inject(&content, &blocks, &stdin);
    fs::write(&cli.output, replaced)?;

    Ok(())
}
