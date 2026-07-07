mod checker;
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

    let output_file = cli.output;
    checker::check_file(&output_file)?;

    let ids = cli.id;
    let lang = detector::detect(&output_file)?;
    let content = fs::read_to_string(&output_file)?;
    let comments = extractor::extract_comments(&content, lang)?;
    let blocks = marker::extract_marker_blocks(&comments)?;
    let stdin = io::read_stdin()?;

    let replaced = injector::inject(&content, &blocks, &stdin, &ids);
    if cli.dry_run {
        println!("{}", replaced);
    } else {
        fs::write(&output_file, replaced)?;
    }

    Ok(())
}
