use std::collections::HashSet;
use std::fs;
use std::io::{self, Read};

use crate::core::inject::inject;
use crate::core::{
    parse::parse_file,
    types::{MarkerBlock, OutputID, Result},
};

pub fn run(input: Option<String>, output: String, dry_run: bool, ids: Vec<OutputID>) -> Result<()> {
    let output_file = parse_file(&output)?;
    let input_blocks = match input {
        Some(i) => {
            let input_file = parse_file(&i)?;
            check_missing_ids(&output_file.blocks, &input_file.blocks)?;
            input_file.blocks
        }
        None => stdin_blocks(ids)?,
    };

    let replaced = inject(&output_file.content, &output_file.blocks, &input_blocks)?;
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

fn check_missing_ids(output_blocks: &[MarkerBlock], input_blocks: &[MarkerBlock]) -> Result<()> {
    let provided: HashSet<&String> = input_blocks.iter().flat_map(|b| &b.input_ids).collect();
    for b in output_blocks {
        if let Some(id) = &b.output_id
            && !provided.contains(id)
        {
            return Err(format!("missing input id `{id}`").into());

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
