use crate::core;
use crate::core::types::{ParsedFile, Result};
use std::fs;

pub fn parse_patterns(patterns: &[String]) -> Result<Vec<ParsedFile>> {
    let mut files = Vec::new();
    for pattern in patterns {
        files.extend(parse_pattern(pattern)?);
    }

    Ok(files)
}

fn parse_file(path: &str) -> Result<ParsedFile> {
    core::checker::check_file(path)?;
    let lang = core::detector::detect(path)?;
    let content = fs::read_to_string(path)?;
    let blocks = core::marker::extract_marker_blocks(&content, path, lang)?;
    Ok(ParsedFile {
        content,
        blocks,
        path: path.to_string(),
    })
}

fn parse_pattern(pattern: &str) -> Result<Vec<ParsedFile>> {
    let mut result: Vec<ParsedFile> = Vec::new();

    // If the pattern is a directory, then parse it recursively.
    let pattern = if std::path::Path::new(pattern).is_dir() {
        format!("{}/**/*", pattern.trim_end_matches('/'))
    } else {
        pattern.to_string()
    };

    for entry in glob::glob(&pattern)? {
        let path = entry?;

        // Ignore directories.
        if path.is_dir() {
            continue;
        }

        let parsed = parse_file(path.to_string_lossy().as_ref())?;
        result.push(parsed);
    }

    if result.is_empty() {
        return Err(format!("no files matched pattern `{pattern}`").into());
    }

    Ok(result)
}
