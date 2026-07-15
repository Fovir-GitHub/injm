use crate::core::types::{BlockRole, Comment, MarkerBlock, Result};

impl MarkerBlock {
    // input block matches output block.
    pub fn matches_output(&self, output: &MarkerBlock) -> bool {
        let BlockRole::Input { ids, .. } = &self.role else {
            return false;
        };

        match &output.role {
            BlockRole::Output { id: Some(id) } => ids.contains(id),
            _ => ids.is_empty(),
        }
    }
}

pub(crate) fn extract_marker_blocks(
    comments: &[Comment],
    content: &str,
    path: &str,
) -> Result<Vec<MarkerBlock>> {
    struct OpenBlock {
        begin_line: usize,
        role: BlockRole,
    }

    let mut marker_blocks: Vec<MarkerBlock> = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut open: Option<OpenBlock> = None;

    for comment in comments {
        if comment.text.contains("injm begin") {
            if open.is_some() {
                return Err(format!(
                    "found nested `injm begin` without `injm end` in line {} of {}",
                    comment.start_line, path,
                )
                .into());
            }
            open = Some(OpenBlock {
                begin_line: comment.end_line,
                role: extract_role(&comment.text)?,
            });
            continue;
        }

        if comment.text.contains("injm end") {
            let OpenBlock { begin_line, role } = open.take().ok_or_else(|| {
                format!(
                    "found `injm end` without `injm begin` in line {}",
                    comment.start_line
                )
            })?;

            let role = match role {
                BlockRole::Input { ids, .. } => {
                    let input_content = lines[begin_line + 1..comment.start_line].join("\n");
                    BlockRole::Input {
                        ids,
                        content: input_content,
                    }
                }
                other => other,
            };

            marker_blocks.push(MarkerBlock {
                begin_line,
                end_line: comment.start_line,
                role,
            });
        }
    }

    if let Some(OpenBlock { begin_line, .. }) = open {
        return Err(format!(
            "found `injm begin` without `injm end` at line {}",
            begin_line
        )
        .into());
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
            return Ok(BlockRole::Default);
        } else {
            return Err("found both input and output token".into());
        }
    }

    if !input_tokens.is_empty() {
        return Ok(BlockRole::Input {
            ids: input_tokens.iter().map(|t| t[1..].to_string()).collect(),
            content: String::from(""),
        });
    }

    match output_tokens.len() {
        1 => Ok(BlockRole::Output {
            id: Some(output_tokens[0][1..].to_string()),
        }),
        _ => Err("multiple output markers detected".into()),
    }
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
        let blocks = extract_marker_blocks(&comments, "", "").unwrap();
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
        let blocks = extract_marker_blocks(&comments, "", "").unwrap();
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
        assert!(extract_marker_blocks(&comments, "", "").is_err());
    }

    #[test]
    fn test_end_without_begin_returns_error() {
        let comments = vec![make_comment("// injm end", 1, 1)];
        assert!(extract_marker_blocks(&comments, "", "").is_err());
    }

    #[test]
    fn test_begin_without_end_returns_error() {
        let comments = vec![make_comment("// injm begin", 1, 1)];
        assert!(extract_marker_blocks(&comments, "", "").is_err());
    }

    #[test]
    fn test_empty_comments() {
        let comments = vec![];
        let blocks = extract_marker_blocks(&comments, "", "").unwrap();
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
        let blocks = extract_marker_blocks(&comments, "", "").unwrap();
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn test_extract_role_default() {
        let role = extract_role("// injm begin").unwrap();
        assert!(matches!(role, BlockRole::Default));
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

    // extract_marker_blocks tests
    #[test]
    fn test_single_block_no_id() {
        let comments = vec![
            make_comment("// injm begin", 1, 1),
            make_comment("// injm end", 5, 5),
        ];
        let blocks = extract_marker_blocks(&comments, "", "").unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(blocks[0].role, BlockRole::Default));
    }

    #[test]
    fn test_single_block_with_output() {
        let comments = vec![
            make_comment("// injm begin >first", 1, 1),
            make_comment("// injm end", 5, 5),
        ];
        let blocks = extract_marker_blocks(&comments, "", "").unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(
            blocks[0].role,
            BlockRole::Output { id: Some(ref id) } if id == "first"
        ));
    }

    #[test]
    fn test_multiple_blocks_ids_dont_leak() {
        let comments = vec![
            make_comment("// injm begin >first", 1, 1),
            make_comment("// injm end", 3, 3),
            make_comment("// injm begin", 5, 5),
            make_comment("// injm end", 7, 7),
        ];
        let blocks = extract_marker_blocks(&comments, "", "").unwrap();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(
            blocks[0].role,
            BlockRole::Output { id: Some(ref id) } if id == "first"
        ));
        assert!(matches!(blocks[1].role, BlockRole::Default));
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
        let blocks = extract_marker_blocks(&comments, content, "").unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(
            blocks[0].role,
            BlockRole::Input { ref ids, .. } if ids == &vec!["hello".to_string()]
        ));
        if let BlockRole::Input { content, .. } = &blocks[0].role {
            assert_eq!(content, "println!(\"Hello injm\")");
        } else {
            panic!("expected Input role");
        }
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
        let blocks = extract_marker_blocks(&comments, content, "").unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(
            blocks[0].role,
            BlockRole::Input { ref ids, .. } if ids == &vec!["hello".to_string()]
        ));
        if let BlockRole::Input { content, .. } = &blocks[0].role {
            assert_eq!(
                content,
                "println!(\"Hello injm\")\nprintln!(\"Hello injm\")\nprintln!(\"Hello injm\")"
            );
        } else {
            panic!("expected Input role");
        }
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
        let blocks = extract_marker_blocks(&comments, content, "").unwrap();
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
        let comments = vec![
            make_comment("// injm begin <first <second", 0, 0),
            make_comment("// injm end", 2, 2),
        ];
        let blocks = extract_marker_blocks(&comments, content, "").unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(
            blocks[0].role,
            BlockRole::Input { ref ids, .. }
                if ids == &vec!["first".to_string(), "second".to_string()]
        ));
        if let BlockRole::Input { content, .. } = &blocks[0].role {
            assert_eq!(content, "old content");
        } else {
            panic!("expected Input role");
        }
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
        let blocks = extract_marker_blocks(&comments, content, "").unwrap();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(blocks[0].role, BlockRole::Input { .. }));
        assert!(matches!(blocks[1].role, BlockRole::Default));
    }
}
