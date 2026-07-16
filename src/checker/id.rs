use crate::types::{BlockRole, MarkerBlock, ParsedFile, Result};
use std::collections::HashSet;

pub fn check_missing_ids(output_files: &[ParsedFile], input_files: &[ParsedFile]) -> Result<()> {
    let input_blocks: HashSet<&String> = input_files
        .iter()
        .flat_map(|file| file.blocks.iter())
        .filter_map(|b| match &b.role {
            BlockRole::Input { ids, .. } => Some(ids.iter()),
            _ => None,
        })
        .flatten()
        .collect();

    if let Some(id) = output_files
        .iter()
        .flat_map(|file| file.blocks.iter())
        .filter_map(|b| match &b.role {
            BlockRole::Output { id } => id.as_ref(),
            _ => None,
        })
        .find(|&id| !input_blocks.contains(id))
    {
        return Err(format!("missing input id `{id}`").into());
    }

    Ok(())
}

pub fn check_duplicated_input_ids(blocks: &[MarkerBlock]) -> Result<()> {
    let mut seen = HashSet::new();

    for block in blocks {
        if let BlockRole::Input { ids, .. } = &block.role {
            for id in ids {
                if !seen.insert(id) {
                    return Err(format!("duplicated input id `{id}`").into());
                }
            }
        }
    }

    Ok(())
}
