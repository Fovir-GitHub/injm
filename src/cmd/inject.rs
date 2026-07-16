use std::fs;
use std::io::{self, Read};

use crate::cli::InjectArgs;
use crate::core::checker::{check_duplicated_input_ids, check_missing_ids};
use crate::core::inject::inject;
use crate::core::parser::parse_patterns;
use crate::core::types::{BlockRole, MarkerBlock, Result};

pub fn run(args: InjectArgs) -> Result<()> {
    let output_files = parse_patterns(&args.output)?;

    let input_blocks: Vec<MarkerBlock> = if args.input.is_empty() {
        stdin_blocks(args.id)?
    } else {
        let input_files = parse_patterns(&args.input)?;
        check_missing_ids(&output_files, &input_files)?;
        input_files
            .into_iter()
            .flat_map(|file| file.blocks)
            .collect()
    };

    check_duplicated_input_ids(&input_blocks)?;

    for output_file in output_files {
        let replaced = inject(&output_file.content, &output_file.blocks, &input_blocks)?;
        if args.dry_run {
            println!("{replaced}");
        } else {
            fs::write(output_file.path, replaced)?;
        }
    }

    Ok(())
}

fn read_stdin() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}

fn stdin_blocks(ids: Vec<Option<String>>) -> Result<Vec<MarkerBlock>> {
    let stdin = read_stdin()?;
    let input_ids = ids.into_iter().flatten().collect();
    Ok(vec![MarkerBlock {
        begin_line: 0,
        end_line: 0,
        role: BlockRole::Input {
            ids: input_ids,
            content: stdin,
        },
    }])
}
