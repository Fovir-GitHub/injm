use std::mem;

use crate::extractor::Comment;
use crate::types::{OutputID, Result};

pub struct MarkerBlock {
    pub begin_line: usize,
    pub end_line: usize,

    // Allow multiple input markers,
    // while a block can have at most one output marker.
    pub input_ids: Option<Vec<String>>,
    pub input_content: Option<String>,
    pub output_id: OutputID,
}

pub fn extract_marker_blocks(comments: &[Comment], content: &str) -> Result<Vec<MarkerBlock>> {
    let mut marker_blocks: Vec<MarkerBlock> = Vec::new();
    let mut begin: Option<usize> = None; // Record the line of `injm begin`.
    let mut input_ids: Option<Vec<String>> = None;
    let mut output_id: OutputID = None;

    for comment in comments {
        // If comment contains `injm begin`, then check nested blocks,
        // and set `begin` to the end line of this commment.
        // If comment contains `injm end`, then check whether `begin` is `None`.
        // If `begin` is `Some`, then push a new block and update assign `None` to begin.
        // Otherwise, return an error for unclosed block.

        if comment.text.contains("injm begin") {
            if begin.is_some() {
                return Err(format!(
                    "found nested `injm begin` without `injm end` in line {}",
                    comment.start_line
                )
                .into());
            }
            begin = Some(comment.end_line);
            (input_ids, output_id) = extract_id(&comment.text)?;
        } else if comment.text.contains("injm end") {
            match begin.take() {
                Some(begin_line) => {
                    let end_line = comment.start_line;

                    // Extract input contents.
                    let input_content = if input_ids.is_some() {
                        let lines: Vec<&str> = content.lines().collect();
                        Some(lines[begin_line + 1..end_line].join("\n"))
                    } else {
                        None
                    };

                    marker_blocks.push(MarkerBlock {
                        begin_line,
                        end_line,
                        input_ids: mem::take(&mut input_ids),
                        output_id: mem::take(&mut output_id),
                        input_content: input_content,
                    });
                }
                None => {
                    return Err(format!(
                        "found `injm end` without `injm begin` in line {}",
                        comment.start_line
                    )
                    .into());
                }
            }
        }
    }

    // Check unclosed `injm begin`.
    match begin {
        Some(b) => Err(format!("found `injm begin` without `injm end` at line {}", b).into()),
        None => Ok(marker_blocks),
    }
}

fn extract_id(comment: &str) -> Result<(Option<Vec<String>>, OutputID)> {
    let input_tokens: Vec<&str> = comment
        .split_whitespace()
        .filter(|t| t.starts_with("<"))
        .collect();
    let output_tokens: Vec<&str> = comment
        .split_whitespace()
        .filter(|t| t.starts_with(">"))
        .collect();

    let input_ids = if input_tokens.is_empty() {
        None
    } else {
        Some(input_tokens.iter().map(|t| t[1..].to_string()).collect())
    };

    let output_id = match output_tokens.len() {
        0 => None,
        1 => Some(output_tokens[0][1..].to_string()),
        _ => return Err("multiple output markers detected".into()),
    };

    Ok((input_ids, output_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_comment(text: &str, start_line: usize, end_line: usize) -> Comment {
        Comment {
            text: text.to_string(),
            start_line,
            end_line,
        }
    }

    #[test]
    fn test_single_block() {
        let comments = vec![
            make_comment("// injm begin", 1, 1),
            make_comment("// injm end", 5, 5),
        ];
        let blocks = extract_marker_blocks(&comments, "").unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].begin_line, 1);
        assert_eq!(blocks[0].end_line, 5);
    }

    #[test]
    fn test_multiple_blocks() {
        let comments = vec![
            make_comment("// injm begin", 1, 1),
            make_comment("// injm end", 3, 3),
            make_comment("// injm begin", 6, 6),
            make_comment("// injm end", 9, 9),
        ];
        let blocks = extract_marker_blocks(&comments, "").unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].begin_line, 1);
        assert_eq!(blocks[0].end_line, 3);
        assert_eq!(blocks[1].begin_line, 6);
        assert_eq!(blocks[1].end_line, 9);
    }

    #[test]
    fn test_nested_begin_returns_error() {
        let comments = vec![
            make_comment("// injm begin", 1, 1),
            make_comment("// injm begin", 3, 3),
        ];
        assert!(extract_marker_blocks(&comments, "").is_err());
    }

    #[test]
    fn test_end_without_begin_returns_error() {
        let comments = vec![make_comment("// injm end", 1, 1)];
        assert!(extract_marker_blocks(&comments, "").is_err());
    }

    #[test]
    fn test_begin_without_end_returns_error() {
        let comments = vec![make_comment("// injm begin", 1, 1)];
        assert!(extract_marker_blocks(&comments, "").is_err());
    }

    #[test]
    fn test_empty_comments() {
        let comments = vec![];
        let blocks = extract_marker_blocks(&comments, "").unwrap();
        assert_eq!(blocks.len(), 0);
    }

    #[test]
    fn test_non_marker_comments_are_ignored() {
        let comments = vec![
            make_comment("// some comment", 1, 1),
            make_comment("// injm begin", 2, 2),
            make_comment("// another comment", 3, 3),
            make_comment("// injm end", 4, 4),
        ];
        let blocks = extract_marker_blocks(&comments, "").unwrap();
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn test_extract_id_no_markers() {
        let (input, output) = extract_id("// injm begin").unwrap();
        assert_eq!(input, None);
        assert_eq!(output, None);
    }

    #[test]
    fn test_extract_id_with_output() {
        let (input, output) = extract_id("// injm begin >first").unwrap();
        assert_eq!(input, None);
        assert_eq!(output, Some("first".to_string()));
    }

    #[test]
    fn test_extract_id_with_input() {
        let (input, output) = extract_id("// injm begin <first <second").unwrap();
        assert_eq!(input, Some(vec!["first".to_string(), "second".to_string()]));
        assert_eq!(output, None);
    }

    #[test]
    fn test_extract_id_with_both() {
        let (input, output) = extract_id("// injm begin <first >second").unwrap();
        assert_eq!(input, Some(vec!["first".to_string()]));
        assert_eq!(output, Some("second".to_string()));
    }

    #[test]
    fn test_extract_id_multiple_outputs_returns_error() {
        assert!(extract_id("// injm begin >first >second").is_err());
    }

    // extract_marker_blocks tests
    #[test]
    fn test_single_block_no_id() {
        let comments = vec![
            make_comment("// injm begin", 1, 1),
            make_comment("// injm end", 5, 5),
        ];
        let blocks = extract_marker_blocks(&comments, "").unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].input_ids, None);
        assert_eq!(blocks[0].output_id, None);
    }

    #[test]
    fn test_single_block_with_output() {
        let comments = vec![
            make_comment("// injm begin >first", 1, 1),
            make_comment("// injm end", 5, 5),
        ];
        let blocks = extract_marker_blocks(&comments, "").unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].output_id, Some("first".to_string()));
    }

    #[test]
    fn test_multiple_blocks_ids_dont_leak() {
        let comments = vec![
            make_comment("// injm begin >first", 1, 1),
            make_comment("// injm end", 3, 3),
            make_comment("// injm begin", 5, 5),
            make_comment("// injm end", 7, 7),
        ];
        let blocks = extract_marker_blocks(&comments, "").unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].output_id, Some("first".to_string()));
        assert_eq!(blocks[1].output_id, None);
    }

    #[test]
    fn test_block_with_input_content() {
        let content = "\
// injm begin <hello
println!(\"Hello injm\")
// injm end";
        let comments = vec![
            make_comment("// injm begin <hello", 0, 0),
            make_comment("// injm end", 2, 2),
        ];
        let blocks = extract_marker_blocks(&comments, content).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].input_ids, Some(vec!["hello".to_string()]));
        assert_eq!(
            blocks[0].input_content,
            Some("println!(\"Hello injm\")".to_string())
        );
    }

    #[test]
    fn test_block_with_multiple_lines_input_content() {
        let content = "\
// injm begin <hello
println!(\"Hello injm\")
println!(\"Hello injm\")
println!(\"Hello injm\")
// injm end";
        let comments = vec![
            make_comment("// injm begin <hello", 0, 0),
            make_comment("// injm end", 4, 4),
        ];
        let blocks = extract_marker_blocks(&comments, content).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].input_ids, Some(vec!["hello".to_string()]));
        assert_eq!(
            blocks[0].input_content,
            Some(
                "println!(\"Hello injm\")\nprintln!(\"Hello injm\")\nprintln!(\"Hello injm\")"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_block_without_input_has_no_content() {
        let content = "\
// injm begin >first
println!(\"Hello\")
// injm end";
        let comments = vec![
            make_comment("// injm begin >first", 0, 0),
            make_comment("// injm end", 2, 2),
        ];
        let blocks = extract_marker_blocks(&comments, content).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].input_ids, None);
        assert_eq!(blocks[0].input_content, None);
    }

    #[test]
    fn test_block_with_multiple_input_ids() {
        let content = "\
// injm begin <first <second
old content
// injm end";
        let comments = vec![
            make_comment("// injm begin <first <second", 0, 0),
            make_comment("// injm end", 2, 2),
        ];
        let blocks = extract_marker_blocks(&comments, content).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0].input_ids,
            Some(vec!["first".to_string(), "second".to_string()])
        );
        assert_eq!(blocks[0].input_content, Some("old content".to_string()));
    }

    #[test]
    fn test_input_content_not_leak_between_blocks() {
        let content = "\
// injm begin <hello
content one
// injm end
// injm begin
content two
// injm end";
        let comments = vec![
            make_comment("// injm begin <hello", 0, 0),
            make_comment("// injm end", 2, 2),
            make_comment("// injm begin", 3, 3),
            make_comment("// injm end", 5, 5),
        ];
        let blocks = extract_marker_blocks(&comments, content).unwrap();
        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].input_content.is_some());
        assert!(blocks[1].input_content.is_none());
    }
}
