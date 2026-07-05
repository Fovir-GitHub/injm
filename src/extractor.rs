use std::{fs, path::Path};

use tree_sitter_language_pack::{ProcessConfig, process};

use crate::error::Result;

pub struct Comment {
    pub text: String,
    pub start_line: usize,
    pub end_line: usize,
}

pub fn extract_comments(path: &Path, lang: &str) -> Result<Vec<Comment>> {
    // Read file contents.
    let content = fs::read_to_string(path)?;
    let mut comments: Vec<Comment> = Vec::new();

    // Query all comments.
    let mut config = ProcessConfig::new(lang);
    config.comments = true;
    let result = process(&content, &config)?;
    for comment in result.comments {
        comments.push(Comment {
            text: comment.text.trim().to_string(),
            start_line: comment.span.start_line,
            end_line: comment.span.end_line,
        });
    }

    Ok(comments)
}
