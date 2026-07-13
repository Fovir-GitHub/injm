use std::collections::HashSet;
use std::fs;
use std::io::{self, Read};

use crate::core::inject::inject;
use crate::core::parser::parse_patterns;
use crate::core::types::ParsedFile;
use crate::core::types::{MarkerBlock, OutputID, Result};

pub fn run(
    input: Vec<String>,
    output: Vec<String>,
    dry_run: bool,
    ids: Vec<OutputID>,
) -> Result<()> {
    let output_files = parse_patterns(&output)?;

    let input_blocks: Vec<MarkerBlock> = if input.is_empty() {
        stdin_blocks(ids)?
    } else {
        let input_files = parse_patterns(&input)?;
        check_missing_ids(&output_files, &input_files)?;
        input_files
            .into_iter()
            .flat_map(|file| file.blocks)
            .collect()
    };

    check_duplicated_ids(&input_blocks)?;

    for output_file in output_files {
        let replaced = inject(&output_file.content, &output_file.blocks, &input_blocks)?;
        if dry_run {
            println!("{replaced}");
        } else {
            fs::write(output_file.path, replaced)?;
        }
    }

    Ok(())
}

fn read_stdin() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}

fn check_missing_ids(output_files: &[ParsedFile], input_files: &[ParsedFile]) -> Result<()> {
    let provided: HashSet<&String> = input_files
        .iter()
        .flat_map(|file| file.blocks.iter())
        .flat_map(|block| block.input_ids.iter())
        .collect();

    if let Some(id) = output_files
        .iter()
        .flat_map(|file| file.blocks.iter())
        .filter_map(|block| block.output_id.as_ref())
        .find(|id| !provided.contains(*id))
    {
        return Err(format!("missing input id `{id}`").into());
    }

    Ok(())
}

fn check_duplicated_ids(blocks: &[MarkerBlock]) -> Result<()> {
    let mut seen = HashSet::new();

    for id in blocks.iter().flat_map(|block| block.input_ids.iter()) {
        if !seen.insert(id) {
            return Err(format!("duplicated input id `{id}`").into());
        }
    }

    Ok(())
}

fn stdin_blocks(ids: Vec<OutputID>) -> Result<Vec<MarkerBlock>> {
    let stdin = read_stdin()?;
    let input_ids = ids.into_iter().flatten().collect();
    Ok(vec![MarkerBlock {
        input_content: Some(stdin),
        begin_line: 0,
        end_line: 0,
        input_ids,
        output_id: None,
    }])
}
