use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use crate::core::inject::inject;
use crate::core::{
    parse::parse_file,
    types::{MarkerBlock, OutputID, Result},
};

pub fn run(
    input: Option<PathBuf>,
    output: PathBuf,
    dry_run: bool,
    ids: Vec<OutputID>,
) -> Result<()> {
    let (output_content, output_blocks) = parse_file(&output)?;
    let input_blocks = match input {
        None => {
            let stdin = read_stdin()?;
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

    let replaced = inject(&output_content, &output_blocks, &input_blocks)?;
    if dry_run {
        println!("{replaced}");
    } else {
        fs::write(output, replaced)?;
    }

    Ok(())
}

fn read_stdin() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}
