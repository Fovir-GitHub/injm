use tree_sitter_language_pack::{ProcessConfig, process};

use crate::error::Result;

pub struct Comment {
    pub text: String,
    pub start_line: usize,
    pub end_line: usize,
}

pub fn extract_comments(content: &str, lang: &str) -> Result<Vec<Comment>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_comments() {
        let content = r#"
fn main() {
    // hello world
    let x = 1;
}
"#;
        let comments = extract_comments(content, "rust").unwrap();
        assert!(comments.iter().any(|c| c.text.contains("hello world")));
    }

    #[test]
    fn test_extract_multiple_comments() {
        let content = r#"
// first comment
fn main() {
    // second comment
}
"#;
        let comments = extract_comments(content, "rust").unwrap();
        assert_eq!(comments.len(), 2);
    }

    #[test]
    fn test_comment_line_numbers() {
        let content = r#"fn main() {
    // hello
}"#;
        let comments = extract_comments(content, "rust").unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].start_line, 1);
        assert_eq!(comments[0].end_line, 1);
    }

    #[test]
    fn test_no_comments() {
        let content = r#"
fn main() {
    let x = 1;
}
"#;
        let comments = extract_comments(content, "rust").unwrap();
        assert_eq!(comments.len(), 0);
    }

    #[test]
    fn test_extract_go_comments() {
        let content = r#"
// first comment
// second comment
// third comment
func main() {
}
"#;
        let comments = extract_comments(content, "go").unwrap();
        assert_eq!(comments.len(), 3);
    }

    #[test]
    fn test_extract_block_comments() {
        let content = r#"
/*
 This is a
 multiple
 line comment
 (aka block comment)
 */
int main(void) {}
"#;
        let comments = extract_comments(content, "c").unwrap();
        assert_eq!(comments.len(), 1);
    }

    #[test]
    fn test_extract_python_comments() {
        let content = r#"
# hello world
def main():
    pass
"#;
        let comments = extract_comments(content, "python").unwrap();
        assert!(comments.iter().any(|c| c.text.contains("hello world")));
    }

    #[test]
    fn test_extract_latex_comments() {
        let content = r#"
\section{injm}

% hello tex
"#;
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
