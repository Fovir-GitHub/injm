mod cli;
mod core;
mod io;

use crate::core::parse::parse_file;
use crate::core::types::{MarkerBlock, Result};
use clap::Parser;
use std::{fs, vec};

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

    let replaced = core::inject::inject(&output_content, &output_blocks, &input_blocks)?;
    if cli.dry_run {
        println!("{}", replaced);
    } else {
        fs::write(&output_file, replaced)?;
    }

    Ok(())
}
