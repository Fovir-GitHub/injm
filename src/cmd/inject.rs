use anyhow::Result;
use std::fs;
use std::io::{self, Read};

use crate::checker::{check_duplicated_input_ids, check_missing_ids};
use crate::cli::InjectArgs;
use crate::injector::inject;
use crate::parser::parse_patterns;
use crate::types::{BlockRole, MarkerBlock, SourceSpan};

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

    let multiple_outputs = output_files.len() > 1;
    for output_file in output_files {
        let replaced = inject(&output_file.content, &output_file.blocks, &input_blocks)?;
        if args.dry_run {
            if multiple_outputs {
                println!("==== {} ====", output_file.path.display());
            }
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
        span: SourceSpan::new(0, 0),
        role: BlockRole::Input {
            ids: input_ids,
            content: stdin,
        },
    }])
}
