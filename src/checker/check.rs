use std::path::PathBuf;

use crate::types::{BlockRole, MarkerBlock, ParsedFile, SourceSpan};

pub struct SyncIssue {
    pub path: PathBuf,
    pub span: SourceSpan,
    pub id: String,
    pub expected: String,
    pub actual: String,
}

pub(crate) fn check_sync(
    input_blocks: &[&MarkerBlock],
    output_files: &[ParsedFile],
) -> Vec<SyncIssue> {
    let mut issues = Vec::new();

    for output_file in output_files {
        for output_block in &output_file.blocks {
            let BlockRole::Output { id: Some(id) } = &output_block.role else {
                continue;
            };

            let Some(input_block) = input_blocks
                .iter()
                .find(|input| input.matches_output(output_block))
            else {
                continue;
            };

            if input_block.content != output_block.content {
                issues.push(SyncIssue {
                    path: output_file.path.clone(),
                    span: output_block.span,
                    id: id.clone(),
                    expected: input_block.content.clone(),
                    actual: output_block.content.clone(),
                });
            }
        }
    }

    issues
}
