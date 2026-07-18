use crate::{
    checker::check_sync,
    cli::CheckArgs,
    cmd::utils::into_blocks,
    output::print_block_diff,
    parser::parse_patterns,
    validator::{validate_duplicated_input_ids, validate_missing_ids},
};
use anyhow::{Result, bail};

pub fn run(args: CheckArgs) -> Result<()> {
    let input_files = parse_patterns(&args.input)?;
    let output_files = parse_patterns(&args.output)?;

    validate_missing_ids(&output_files, &input_files)?;

    let input_blocks = into_blocks(input_files);
    validate_duplicated_input_ids(&input_blocks)?;

    let issues = check_sync(&input_blocks, &output_files);
    if issues.is_empty() {
        println!("all marker blocks are synchronized");
        return Ok(());
    }

    for issue in &issues {
        eprintln!(
            "{}:{}: output block `{}` is out of sync",
            issue.path.display(),
            issue.span.display_lines(),
            issue.id
        );

        if args.diff {
            print_block_diff(
                &issue.path,
                &issue.span.display_lines(),
                &issue.id,
                &issue.actual,
                &issue.expected,
            );
        }
    }

    bail!("{} output block(s) are out of sync", issues.len());
}
