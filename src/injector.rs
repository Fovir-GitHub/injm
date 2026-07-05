use crate::marker::MarkerBlock;

pub fn inject(content: &str, blocks: &[MarkerBlock], stdin: &str) -> String {
    let mut lines: Vec<&str> = content.lines().collect();

    // Use reversed iteration to avoid changes of line number.
    for block in blocks.iter().rev() {
        // Content to be replaced is in (begin_line, end_line).
        let before = lines[..=block.begin_line].to_vec();
        let after = lines[block.end_line..].to_vec();

        lines = Vec::new();
        lines.extend_from_slice(&before);
        lines.push(&stdin);
        lines.extend_from_slice(&after);
    }

    let mut result = lines.join("\n");
    if content.ends_with("\n") {
        result.push('\n');
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marker::MarkerBlock;

    #[test]
    fn test_inject_single_block() {
        let content = "fn main() {\n    // injm begin\n    old content\n    // injm end\n}\n";
        let blocks = vec![MarkerBlock {
            begin_line: 1,
            end_line: 3,
        }];
        let result = inject(content, &blocks, "    new content");
        assert!(result.contains("new content"));
        assert!(!result.contains("old content"));
    }

    #[test]
    fn test_inject_preserves_markers() {
        let content = "// injm begin\nold\n// injm end\n";
        let blocks = vec![MarkerBlock {
            begin_line: 0,
            end_line: 2,
        }];
        let result = inject(content, &blocks, "new");
        assert!(result.contains("// injm begin"));
        assert!(result.contains("// injm end"));
    }

    #[test]
    fn test_inject_preserves_trailing_newline() {
        let content = "// injm begin\nold\n// injm end\n";
        let blocks = vec![MarkerBlock {
            begin_line: 0,
            end_line: 2,
        }];
        let result = inject(content, &blocks, "new");
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_inject_no_trailing_newline() {
        let content = "// injm begin\nold\n// injm end";
        let blocks = vec![MarkerBlock {
            begin_line: 0,
            end_line: 2,
        }];
        let result = inject(content, &blocks, "new");
        assert!(!result.ends_with('\n'));
    }

    #[test]
    fn test_inject_multiple_blocks() {
        let content =
            "// injm begin\nold one\n// injm end\ncode\n// injm begin\nold two\n// injm end\n";
        let blocks = vec![
            MarkerBlock {
                begin_line: 0,
                end_line: 2,
            },
            MarkerBlock {
                begin_line: 4,
                end_line: 6,
            },
        ];
        let result = inject(content, &blocks, "new");
        assert!(!result.contains("old one"));
        assert!(!result.contains("old two"));
        assert_eq!(result.matches("new").count(), 2);
    }

    #[test]
    fn test_inject_empty_block() {
        let content = "// injm begin\n// injm end\n";
        let blocks = vec![MarkerBlock {
            begin_line: 0,
            end_line: 1,
        }];
        let result = inject(content, &blocks, "new content");
        assert!(result.contains("new content"));
    }

    #[test]
    fn test_inject_multiline_stdin() {
        let content = "// injm begin\nold\n// injm end\n";
        let blocks = vec![MarkerBlock {
            begin_line: 0,
            end_line: 2,
        }];
        let result = inject(content, &blocks, "line one\nline two\nline three");
        assert!(result.contains("line one\nline two\nline three"));
    }
}
