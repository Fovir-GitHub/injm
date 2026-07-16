use crate::cli::ListArgs;
use crate::output::print;
use crate::parser::parse_patterns;
use crate::types::{BlockRole, MarkerInfo, MarkerType};
use anyhow::Result;

pub fn run(args: ListArgs) -> Result<()> {
    // If the input is empty, then fallback to current directory (`.`)
    let input: Vec<String> = if args.input.is_empty() {
        vec![".".to_string()]
    } else {
        args.input
    };

    // Get all input and output blocks.
    let mut rows = Vec::new();

    let files = parse_patterns(&input)?;
    for file in &files {
        for block in &file.blocks {
            match &block.role {
                BlockRole::Input { ids, .. } => {
                    for id in ids {
                        rows.push(MarkerInfo {
                            file: file.path.clone(),
                            marker_type: MarkerType::Input,
                            id: id.clone(),
                            lines: format!("{}-{}", block.begin_line + 1, block.end_line + 1),
                        });
                    }
                }
                BlockRole::Output { id } => {
                    if let Some(id) = id {
                        rows.push(MarkerInfo {
                            file: file.path.clone(),
                            marker_type: MarkerType::Output,
                            id: id.clone(),
                            lines: format!("{}-{}", block.begin_line + 1, block.end_line + 1),
                        });
                    }
                }
                BlockRole::Default => {}
            }
        }
    }

    // Display input and output blocks, including
    // Path, ID, input/output.
    print(&rows, args.format)?;

    Ok(())
}
