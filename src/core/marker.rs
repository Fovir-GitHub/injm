use crate::core::types::{BlockRole, MarkerBlock, Result};
use tree_sitter_language_pack::{ProcessConfig, process};

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

struct Comment {
    pub text: String,
    pub start_line: usize,
    pub end_line: usize,
}

pub(crate) fn extract_marker_blocks(
    content: &str,
    path: &str,
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

fn extract_comments(content: &str, lang: &str) -> Result<Vec<Comment>> {
    let mut comments: Vec<Comment> = Vec::new();

    // Query all comments.
    let mut config = ProcessConfig::new(lang);
    config.comments = true;
    let result = process(content, &config)?;
    for comment in result.comments {
        comments.push(Comment {
            text: comment.text.trim().to_string(),
            start_line: comment.span.start_line,
            end_line: comment.span.end_line,
        });
    }

    Ok(comments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_block() {
        let content = "\n// injm begin\n\n\n\n// injm end\n";
        let blocks = extract_marker_blocks(content, "", "rust").unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].begin_line, 1);
        assert_eq!(blocks[0].end_line, 5);
    }

    #[test]
    fn test_multiple_blocks() {
        let content = "\n// injm begin\n\n// injm end\n\n\n// injm begin\n\n\n// injm end\n";
        let blocks = extract_marker_blocks(content, "", "rust").unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].begin_line, 1);
        assert_eq!(blocks[0].end_line, 3);
        assert_eq!(blocks[1].begin_line, 6);
        assert_eq!(blocks[1].end_line, 9);
    }

    #[test]
    fn test_nested_begin_returns_error() {
        let content = "\n// injm begin\n\n// injm begin\n";
        assert!(extract_marker_blocks(content, "", "rust").is_err());
    }

    #[test]
    fn test_end_without_begin_returns_error() {
        let content = "\n// injm end\n";
        assert!(extract_marker_blocks(content, "", "rust").is_err());
    }

    #[test]
    fn test_begin_without_end_returns_error() {
        let content = "\n// injm begin\n";
        assert!(extract_marker_blocks(content, "", "rust").is_err());
    }

    #[test]
    fn test_empty_comments() {
        let blocks = extract_marker_blocks("fn main() {}", "", "rust").unwrap();
        assert_eq!(blocks.len(), 0);
    }

    #[test]
    fn test_non_marker_comments_are_ignored() {
        let content = "\n// some comment\n// injm begin\n// another comment\n// injm end\n";
        let blocks = extract_marker_blocks(content, "", "rust").unwrap();
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

    #[test]
    fn test_single_block_no_id() {
        let content = "\n// injm begin\n\n\n\n// injm end\n";
        let blocks = extract_marker_blocks(content, "", "rust").unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(blocks[0].role, BlockRole::Default));
    }

    #[test]
    fn test_single_block_with_output() {
        let content = "\n// injm begin >first\n\n\n\n// injm end\n";
        let blocks = extract_marker_blocks(content, "", "rust").unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(
            blocks[0].role,
            BlockRole::Output { id: Some(ref id) } if id == "first"
        ));
    }

    #[test]
    fn test_multiple_blocks_ids_dont_leak() {
        let content = "\n// injm begin >first\n\n// injm end\n\n\n// injm begin\n\n// injm end\n";
        let blocks = extract_marker_blocks(content, "", "rust").unwrap();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(
            blocks[0].role,
            BlockRole::Output { id: Some(ref id) } if id == "first"
        ));
        assert!(matches!(blocks[1].role, BlockRole::Default));
    }

    #[test]
    fn test_block_with_input_content() {
        let content = "// injm begin <hello\nprintln!(\"Hello injm\")\n// injm end";
        let blocks = extract_marker_blocks(content, "", "rust").unwrap();
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
        let blocks = extract_marker_blocks(content, "", "rust").unwrap();
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
        let content = "// injm begin >first\nprintln!(\"Hello\")\n// injm end";
        let blocks = extract_marker_blocks(content, "", "rust").unwrap();
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
        let blocks = extract_marker_blocks(content, "", "rust").unwrap();
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
        let blocks = extract_marker_blocks(content, "", "rust").unwrap();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(blocks[0].role, BlockRole::Input { .. }));
        assert!(matches!(blocks[1].role, BlockRole::Default));
    }

    #[test]
    fn test_extract_rust_comments() {
        let content = r"
fn main() {
    // hello world
    let x = 1;
}
";
        let comments = extract_comments(content, "rust").unwrap();
        assert!(comments.iter().any(|c| c.text.contains("hello world")));
    }

    #[test]
    fn test_extract_multiple_comments() {
        let content = r"
// first comment
fn main() {
    // second comment
}
";
        let comments = extract_comments(content, "rust").unwrap();
        assert_eq!(comments.len(), 2);
    }

    #[test]
    fn test_comment_line_numbers() {
        let content = r"fn main() {
    // hello
}";
        let comments = extract_comments(content, "rust").unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].start_line, 1);
        assert_eq!(comments[0].end_line, 1);
    }

    #[test]
    fn test_no_comments() {
        let content = r"
fn main() {
    let x = 1;
}
";
        let comments = extract_comments(content, "rust").unwrap();
        assert_eq!(comments.len(), 0);
    }

    #[test]
    fn test_extract_go_comments() {
        let content = r"
// first comment
// second comment
// third comment
func main() {
}
";
        let comments = extract_comments(content, "go").unwrap();
        assert_eq!(comments.len(), 3);
    }

    #[test]
    fn test_extract_block_comments() {
        let content = r"
/*
 This is a
 multiple
 line comment
 (aka block comment)
 */
int main(void) {}
";
        let comments = extract_comments(content, "c").unwrap();
        assert_eq!(comments.len(), 1);
    }

    #[test]
    fn test_extract_python_comments() {
        let content = r"
# hello world
def main():
    pass
";
        let comments = extract_comments(content, "python").unwrap();
        assert!(comments.iter().any(|c| c.text.contains("hello world")));
    }

    #[test]
    fn test_extract_latex_comments() {
        let content = r"
\section{injm}

% hello tex
";
        let comments = extract_comments(content, "latex").unwrap();
        assert!(comments.iter().any(|c| c.text.contains("hello tex")));
    }

    #[test]
    fn test_string_is_not_comment() {
        let content = r#"
fn main() {
    let x = "// this is not a comment";
}
"#;
        let comments = extract_comments(content, "rust").unwrap();
        assert_eq!(comments.len(), 0);
    }
}
