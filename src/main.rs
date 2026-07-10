mod checker;
mod cli;
mod detector;
mod extractor;
mod injector;
mod io;
mod marker;
mod types;

use crate::{extractor::extract_comments, marker::MarkerBlock, types::Result};
use clap::Parser;
use std::{fs, path::Path, vec};

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    let output_file = cli.output;
    let ids = cli.id;
    let (output_content, output_blocks) = parse_file(&output_file)?;

    let input_blocks = match cli.input {
        None => {
            let stdin = io::read_stdin()?;
            // Set input_ids to ids.
            let input_ids: Vec<String> = ids
                .iter()
                .map(|id| id.clone().unwrap_or_default())
                .collect();

            vec![MarkerBlock {
                input_content: Some(stdin),
                begin_line: 0,
                end_line: 0,
                input_ids: input_ids,
                output_id: None,
            }]
        }
        Some(input_file) => {
            let (_, block) = parse_file(&input_file)?;
            block
        }
    };

    let replaced = injector::inject(&output_content, &output_blocks, &input_blocks)?;
    if cli.dry_run {
        println!("{}", replaced);
    } else {
        fs::write(&output_file, replaced)?;
    }

    Ok(())
}

fn parse_file(path: &Path) -> Result<(String, Vec<MarkerBlock>)> {
    checker::check_file(path)?;
    let lang = detector::detect(path)?;
    let content = fs::read_to_string(path)?;
    let comments = extract_comments(&content, lang)?;
    let blocks = marker::extract_marker_blocks(&comments, &content)?;
    Ok((content, blocks))
}
