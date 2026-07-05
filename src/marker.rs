use crate::error::Result;
use crate::extractor::Comment;

pub struct MarkerBlock {
    pub begin_line: usize,
    pub end_line: usize,
}

pub fn extract_marker_blocks(comments: &[Comment]) -> Result<Vec<MarkerBlock>> {
    let mut marker_blocks: Vec<MarkerBlock> = Vec::new();
    let mut begin: Option<usize> = None; // Record the line of `injm begin`.

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
        } else if comment.text.contains("injm end") {
            match begin.take() {
                Some(b) => marker_blocks.push(MarkerBlock {
                    begin_line: b,
                    end_line: comment.start_line,
                }),
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
        let blocks = extract_marker_blocks(&comments).unwrap();
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
        let blocks = extract_marker_blocks(&comments).unwrap();
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
        assert!(extract_marker_blocks(&comments).is_err());
    }

    #[test]
    fn test_end_without_begin_returns_error() {
        let comments = vec![make_comment("// injm end", 1, 1)];
        assert!(extract_marker_blocks(&comments).is_err());
    }

    #[test]
    fn test_begin_without_end_returns_error() {
        let comments = vec![make_comment("// injm begin", 1, 1)];
        assert!(extract_marker_blocks(&comments).is_err());
    }

    #[test]
    fn test_empty_comments() {
        let comments = vec![];
        let blocks = extract_marker_blocks(&comments).unwrap();
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
        let blocks = extract_marker_blocks(&comments).unwrap();
        assert_eq!(blocks.len(), 1);
    }
}
