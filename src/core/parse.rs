use crate::core;
use crate::core::types::{MarkerBlock, Result};
use std::fs;
use std::path::Path;

pub fn parse_file(path: &Path) -> Result<(String, Vec<MarkerBlock>)> {
    core::checker::check_file(path)?;
    let lang = core::detector::detect(path)?;
    let content = fs::read_to_string(path)?;
    let comments = core::extractor::extract_comments(&content, lang)?;
    let blocks = core::marker::extract_marker_blocks(&comments, &content)?;
    Ok((content, blocks))
}
