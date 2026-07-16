use super::{ParserError, Result};
use crate::{
    parser::{detector::detect, marker::extract_marker_blocks},
    types::ParsedFile,
    validator::validate_file,
};
use std::{fs, path::Path};

pub fn parse_patterns(patterns: &[String]) -> Result<Vec<ParsedFile>> {
    let mut files = Vec::new();
    for pattern in patterns {
        files.extend(parse_pattern(pattern)?);
    }

    Ok(files)
}

fn parse_file(path: &Path) -> Result<ParsedFile> {
    validate_file(path)?;
    let lang = detect(path)?;
    let content = fs::read_to_string(path)?;
    let blocks = extract_marker_blocks(&content, path, lang)?;
    Ok(ParsedFile {
        content,
        blocks,
        path: path.to_path_buf(),
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

        let parsed = parse_file(&path)?;
        result.push(parsed);
    }

    if result.is_empty() {
        return Err(ParserError::NoPatternMatch { pattern });
    }

    Ok(result)
}
