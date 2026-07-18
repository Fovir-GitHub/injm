use crate::types::{MarkerBlock, ParsedFile};

pub(super) fn into_blocks(files: Vec<ParsedFile>) -> Vec<MarkerBlock> {
    files.into_iter().flat_map(|f| f.blocks).collect()
}
