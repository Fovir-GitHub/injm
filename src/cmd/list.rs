use crate::core::parser::parse_patterns;
use crate::core::types::{MarkerInfo, MarkerType, Result};
use crate::output::OutputFormat;
use crate::output::print;

pub fn run(input: Vec<String>, format: OutputFormat) -> Result<()> {
    // If the input is empty, then fallback to current directory (`.`)
    let input: Vec<String> = if input.is_empty() {
        vec![".".to_string()]
    } else {
        input
    };

    // Get all input and output blocks.
    let mut rows = Vec::new();

    let files = parse_patterns(&input)?;
    for file in &files {
        for block in &file.blocks {
            for id in &block.input_ids {
                rows.push(MarkerInfo {
                    file: file.path.clone(),
                    marker_type: MarkerType::Input,
                    id: id.clone(),
                    lines: format!("{}-{}", block.begin_line + 1, block.end_line + 1),
                });
            }

            if let Some(id) = &block.output_id {
                rows.push(MarkerInfo {
                    file: file.path.clone(),
                    marker_type: MarkerType::Output,
                    id: id.clone(),
                    lines: format!("{}-{}", block.begin_line + 1, block.end_line + 1),
                });
            }
        }
    }

    // Display input and output blocks, including
    // Path, ID, input/output.
    print(&rows, format)?;

    Ok(())
}
