use super::Result;
use crate::types::{BlockRole, MarkerBlock, ParsedFile};
use crate::validator::ValidatorError;
use std::collections::HashSet;

pub fn validate_missing_ids(output_files: &[ParsedFile], input_files: &[ParsedFile]) -> Result<()> {
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
        return Err(ValidatorError::MissingInputID { id: id.to_owned() });
    }

    Ok(())
}

pub fn validate_duplicated_input_ids<'a>(
    blocks: impl IntoIterator<Item = &'a MarkerBlock>,
) -> Result<()> {
    let mut seen: HashSet<&'a str> = HashSet::new();

    for block in blocks {
        let BlockRole::Input { ids, .. } = &block.role else {
            continue;
        };

        for id in ids {
            if !seen.insert(id) {
                return Err(ValidatorError::DuplicatedInputID { id: id.to_owned() });
            }
        }
    }

    Ok(())
}
