use crate::{marker::MarkerBlock, types::OutputID};

pub fn inject(
    content: &str,
    blocks: &[MarkerBlock],
    stdin: &str,
    target_ids: &[OutputID],
) -> String {
    let mut lines: Vec<&str> = content.lines().collect();

    // Use reversed iteration to avoid changes of line number.
    for block in blocks.iter().rev() {
        let should_inject = if target_ids.is_empty() {
            block.output_id.is_none()
        } else {
            target_ids.contains(&block.output_id)
        };

        if should_inject {
            lines = inject_into_a_block(lines, block, stdin);
        }
    }

    let mut result = lines.join("\n");
    if content.ends_with("\n") {
        result.push('\n');
    }

    result
}

fn inject_into_a_block<'a>(
    lines: Vec<&'a str>,
    block: &MarkerBlock,
    stdin: &'a str,
) -> Vec<&'a str> {
    // Content to be replaced is in (begin_line, end_line).
    let before = lines[..=block.begin_line].to_vec();
    let after = lines[block.end_line..].to_vec();

    let mut injected = Vec::new();
    injected.extend_from_slice(&before);
    injected.push(stdin);
    injected.extend_from_slice(&after);

    injected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marker::MarkerBlock;

    #[test]
    fn test_inject_single_block() {
        let content = "fn main() {\n    // injm begin\n    old content\n    // injm end\n}\n";
        let blocks = vec![MarkerBlock {
            input_ids: None,
            output_id: None,
            begin_line: 1,
            end_line: 3,
        }];
        let result = inject(content, &blocks, "    new content", &[None]);
        assert!(result.contains("new content"));
        assert!(!result.contains("old content"));
    }

    #[test]
    fn test_inject_preserves_markers() {
        let content = "// injm begin\nold\n// injm end\n";
        let blocks = vec![MarkerBlock {
            input_ids: None,
            output_id: None,
            begin_line: 0,
            end_line: 2,
        }];
        let result = inject(content, &blocks, "new", &[None]);
        assert!(result.contains("// injm begin"));
        assert!(result.contains("// injm end"));
    }

    #[test]
    fn test_inject_preserves_trailing_newline() {
        let content = "// injm begin\nold\n// injm end\n";
        let blocks = vec![MarkerBlock {
            input_ids: None,
            output_id: None,
            begin_line: 0,
            end_line: 2,
        }];
        let result = inject(content, &blocks, "new", &[None]);
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_inject_no_trailing_newline() {
        let content = "// injm begin\nold\n// injm end";
        let blocks = vec![MarkerBlock {
            input_ids: None,
            output_id: None,
            begin_line: 0,
            end_line: 2,
        }];
        let result = inject(content, &blocks, "new", &[None]);
        assert!(!result.ends_with('\n'));
    }

    #[test]
    fn test_inject_multiple_blocks() {
        let content =
            "// injm begin\nold one\n// injm end\ncode\n// injm begin\nold two\n// injm end\n";
        let blocks = vec![
            MarkerBlock {
                input_ids: None,
                output_id: None,
                begin_line: 0,
                end_line: 2,
            },
            MarkerBlock {
                input_ids: None,
                output_id: None,
                begin_line: 4,
                end_line: 6,
            },
        ];
        let result = inject(content, &blocks, "new", &[None]);
        assert!(!result.contains("old one"));
        assert!(!result.contains("old two"));
        assert_eq!(result.matches("new").count(), 2);
    }

    #[test]
    fn test_inject_empty_block() {
        let content = "// injm begin\n// injm end\n";
        let blocks = vec![MarkerBlock {
            input_ids: None,
            output_id: None,
            begin_line: 0,
            end_line: 1,
        }];
        let result = inject(content, &blocks, "new content", &[None]);
        assert!(result.contains("new content"));
    }

    #[test]
    fn test_inject_multiline_stdin() {
        let content = "// injm begin\nold\n// injm end\n";
        let blocks = vec![MarkerBlock {
            input_ids: None,
            output_id: None,
            begin_line: 0,
            end_line: 2,
        }];
        let result = inject(content, &blocks, "line one\nline two\nline three", &[None]);
        assert!(result.contains("line one\nline two\nline three"));
    }

    #[test]
    fn test_inject_with_id() {
        let content = "\
// injm begin :first
old first
// injm end :first
// injm begin :second
old second
// injm end :second
";
        let blocks = vec![
            MarkerBlock {
                input_ids: None,
                output_id: Some("first".to_string()),
                begin_line: 0,
                end_line: 2,
            },
            MarkerBlock {
                input_ids: None,
                output_id: Some("second".to_string()),
                begin_line: 3,
                end_line: 5,
            },
        ];
        let result = inject(
            content,
            &blocks,
            "new content",
            &[Some("first".to_string())],
        );
        assert!(result.contains("new content"));
        assert!(!result.contains("old first"));
        assert!(result.contains("old second"));
    }

    #[test]
    fn test_inject_when_no_ids() {
        let content = "\
// injm begin :first
old first
// injm end :first
// injm begin :second
old second
// injm end :second
// injm begin
// injm end
";
        let blocks = vec![
            MarkerBlock {
                input_ids: None,
                output_id: Some("first".to_string()),
                begin_line: 0,
                end_line: 2,
            },
            MarkerBlock {
                input_ids: None,
                output_id: Some("second".to_string()),
                begin_line: 3,
                end_line: 5,
            },
            MarkerBlock {
                input_ids: None,
                output_id: None,
                begin_line: 6,
                end_line: 8,
            },
        ];
        let result = inject(content, &blocks, "new content", &[]);
        assert!(result.contains("old first"));
        assert!(result.contains("old second"));
        assert_eq!(result.matches("new content").count(), 1);
    }

    #[test]
    fn test_inject_multiple_ids() {
        let content = "\
// injm begin :first
old first
// injm end :first
// injm begin :second
old second
// injm end :second
// injm begin :third
old third
// injm end :third
";
        let blocks = vec![
            MarkerBlock {
                input_ids: None,
                output_id: Some("first".to_string()),
                begin_line: 0,
                end_line: 2,
            },
            MarkerBlock {
                input_ids: None,
                output_id: Some("second".to_string()),
                begin_line: 3,
                end_line: 5,
            },
            MarkerBlock {
                input_ids: None,
                output_id: Some("third".to_string()),
                begin_line: 6,
                end_line: 8,
            },
        ];
        let result = inject(
            content,
            &blocks,
            "new content",
            &[Some("first".to_string()), Some("third".to_string())],
        );
        assert!(!result.contains("old first"));
        assert!(result.contains("old second"));
        assert!(!result.contains("old third"));
        assert_eq!(result.matches("new content").count(), 2);
    }
}
