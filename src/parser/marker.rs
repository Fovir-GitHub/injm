use std::path::Path;

use super::comment::extract_comments;
use super::{ParserError, Result};
use crate::types::{BlockRole, MarkerBlock, SourceSpan};

impl MarkerBlock {
    // input block matches output block.
    pub fn matches_output(&self, output: &MarkerBlock) -> bool {
        let BlockRole::Input { ids, .. } = &self.role else {
            return false;
        };

        match &output.role {
            BlockRole::Output { id } => {
                if let Some(id) = id {
                    ids.contains(id)
                } else {
                    ids.is_empty()
                }
            }
            _ => false,
        }
    }
}

pub(crate) fn extract_marker_blocks(
    content: &str,
    path: &Path,
    lang: &str,
) -> Result<Vec<MarkerBlock>> {
    struct OpenBlock {
        begin_line: usize,
        role: BlockRole,
    }

    let mut marker_blocks: Vec<MarkerBlock> = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut open: Option<OpenBlock> = None;
    let comments = extract_comments(content, lang)?;

    for comment in comments {
        if comment.text.contains("injm begin") {
            if open.is_some() {
                return Err(ParserError::NestedMarker {
                    line: comment.start_line,
                    path: path.to_owned(),
                });
            }
            open = Some(OpenBlock {
                begin_line: comment.end_line,
                role: extract_role(&comment.text)?,
            });
            continue;
        }

        if comment.text.contains("injm end") {
            let OpenBlock { begin_line, role } =
                open.take().ok_or_else(|| ParserError::UnclosedMarker {
                    line: comment.start_line,
                    path: path.to_owned(),
                })?;

            let span = SourceSpan::new(begin_line, comment.start_line);
            let content = lines[span.content_lines()].join("\n");

            let role = match role {
                BlockRole::Input { ids } => BlockRole::Input { ids },
                other => other,
            };

            marker_blocks.push(MarkerBlock {
                span,
                role,
                content,
            });
        }
    }

    if let Some(OpenBlock { begin_line, .. }) = open {
        return Err(ParserError::UnclosedMarker {
            line: begin_line,
            path: path.to_owned(),
        });
    }

    Ok(marker_blocks)
}

fn extract_role(comment: &str) -> Result<BlockRole> {
    let input_tokens: Vec<&str> = comment
        .split_whitespace()
        .filter(|t| t.starts_with('<'))
        .collect();
    let output_tokens: Vec<&str> = comment
        .split_whitespace()
        .filter(|t| t.starts_with('>'))
        .collect();

    if input_tokens.is_empty() == output_tokens.is_empty() {
        if input_tokens.is_empty() {
            return Ok(BlockRole::Output { id: None });
        } else {
            return Err(ParserError::BothInputOutputMarker {
                comment: comment.to_owned(),
            });
        }
    }

    if !input_tokens.is_empty() {
        return Ok(BlockRole::Input {
            ids: input_tokens.iter().map(|t| t[1..].to_string()).collect(),
        });
    }

    match output_tokens.len() {
        1 => Ok(BlockRole::Output {
            id: Some(output_tokens[0][1..].to_string()),
        }),
        _ => Err(ParserError::MultipleOutputMarker {
            comment: comment.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_block() {
        let content = "\n// injm begin\n\n\n\n// injm end\n";
        let blocks = extract_marker_blocks(content, Path::new(""), "rust").unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0].span,
            SourceSpan {
                begin_marker: 1,
                end_marker: 5
            }
        );
    }

    #[test]
    fn test_multiple_blocks() {
        let content = "\n// injm begin\n\n// injm end\n\n\n// injm begin\n\n\n// injm end\n";
        let blocks = extract_marker_blocks(content, Path::new(""), "rust").unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(
            blocks[0].span,
            SourceSpan {
                begin_marker: 1,
                end_marker: 3
            }
        );
        assert_eq!(
            blocks[1].span,
            SourceSpan {
                begin_marker: 6,
                end_marker: 9
            }
        );
    }

    #[test]
    fn test_nested_begin_returns_error() {
        let content = "\n// injm begin\n\n// injm begin\n";
        assert!(extract_marker_blocks(content, Path::new(""), "rust").is_err());
    }

    #[test]
    fn test_end_without_begin_returns_error() {
        let content = "\n// injm end\n";
        assert!(extract_marker_blocks(content, Path::new(""), "rust").is_err());
    }

    #[test]
    fn test_begin_without_end_returns_error() {
        let content = "\n// injm begin\n";
        assert!(extract_marker_blocks(content, Path::new(""), "rust").is_err());
    }

    #[test]
    fn test_empty_comments() {
        let blocks = extract_marker_blocks("fn main() {}", Path::new(""), "rust").unwrap();
        assert_eq!(blocks.len(), 0);
    }

    #[test]
    fn test_non_marker_comments_are_ignored() {
        let content = "\n// some comment\n// injm begin\n// another comment\n// injm end\n";
        let blocks = extract_marker_blocks(content, Path::new(""), "rust").unwrap();
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn test_extract_role_default() {
        let role = extract_role("// injm begin").unwrap();
        assert!(matches!(role, BlockRole::Output { .. }));
    }

    #[test]
    fn test_extract_role_output() {
        let role = extract_role("// injm begin >first").unwrap();
        assert!(matches!(role, BlockRole::Output { id: Some(ref id) } if id == "first"));
    }

    #[test]
    fn test_extract_role_input() {
        let role = extract_role("// injm begin <first <second").unwrap();
        assert!(
            matches!(role, BlockRole::Input { ref ids, .. } if ids == &vec!["first".to_string(), "second".to_string()])
        );
    }

    #[test]
    fn test_extract_role_both_input_and_output_returns_error() {
        assert!(extract_role("// injm begin <first >second").is_err());
    }

    #[test]
    fn test_extract_role_multiple_outputs_returns_error() {
        assert!(extract_role("// injm begin >first >second").is_err());
    }

    #[test]
    fn test_single_block_no_id() {
        let content = "\n// injm begin\n\n\n\n// injm end\n";
        let blocks = extract_marker_blocks(content, Path::new(""), "rust").unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(blocks[0].role, BlockRole::Output { .. }));
    }

    #[test]
    fn test_single_block_with_output() {
        let content = "\n// injm begin >first\n\n\n\n// injm end\n";
        let blocks = extract_marker_blocks(content, Path::new(""), "rust").unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(
            blocks[0].role,
            BlockRole::Output { id: Some(ref id) } if id == "first"
        ));
    }

    #[test]
    fn test_multiple_blocks_ids_dont_leak() {
        let content = "\n// injm begin >first\n\n// injm end\n\n\n// injm begin\n\n// injm end\n";
        let blocks = extract_marker_blocks(content, Path::new(""), "rust").unwrap();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(
            blocks[0].role,
            BlockRole::Output { id: Some(ref id) } if id == "first"
        ));
        assert!(matches!(blocks[1].role, BlockRole::Output { .. }));
    }

    #[test]
    fn test_block_with_input_content() {
        let content = "// injm begin <hello\nprintln!(\"Hello injm\")\n// injm end";
        let blocks = extract_marker_blocks(content, Path::new(""), "rust").unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(
            blocks[0].role,
            BlockRole::Input { ref ids, .. } if ids == &vec!["hello".to_string()]
        ));

        assert_eq!(blocks[0].content, "println!(\"Hello injm\")");
    }

    #[test]
    fn test_block_with_multiple_lines_input_content() {
        let content = "\
// injm begin <hello
println!(\"Hello injm\")
println!(\"Hello injm\")
println!(\"Hello injm\")
// injm end";
        let blocks = extract_marker_blocks(content, Path::new(""), "rust").unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(
            blocks[0].role,
            BlockRole::Input { ref ids, .. } if ids == &vec!["hello".to_string()]
        ));
        assert_eq!(
            blocks[0].content,
            "println!(\"Hello injm\")\nprintln!(\"Hello injm\")\nprintln!(\"Hello injm\")"
        );
    }

    #[test]
    fn test_block_without_input_has_no_content() {
        let content = "// injm begin >first\nprintln!(\"Hello\")\n// injm end";
        let blocks = extract_marker_blocks(content, Path::new(""), "rust").unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(
            blocks[0].role,
            BlockRole::Output { id: Some(ref id) } if id == "first"
        ));
    }

    #[test]
    fn test_block_with_multiple_input_ids() {
        let content = "\
// injm begin <first <second
old content
// injm end";
        let blocks = extract_marker_blocks(content, Path::new(""), "rust").unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(
            blocks[0].role,
            BlockRole::Input { ref ids, .. }
                if ids == &vec!["first".to_string(), "second".to_string()]
        ));
        assert_eq!(blocks[0].content, "old content");
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
        let blocks = extract_marker_blocks(content, Path::new(""), "rust").unwrap();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(blocks[0].role, BlockRole::Input { .. }));
        assert!(matches!(blocks[1].role, BlockRole::Output { .. }));
    }
}
