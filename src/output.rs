use std::path::Path;

use clap::ValueEnum;
use serde::Serialize;
use similar::TextDiff;
use tabled::{Table, Tabled};

use anyhow::Result;

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

pub fn print<T>(rows: &[T], format: OutputFormat) -> Result<()>
where
    T: Serialize + Tabled,
{
    match format {
        OutputFormat::Table => {
            println!("{}", Table::new(rows));
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(rows)?);
        }
    }

    Ok(())
}

pub fn print_diff(path: &Path, original: &str, replaced: &str) {
    let old_path = format!("a/{}", path.display());
    let new_path = format!("b/{}", path.display());

    println!(
        "{}",
        TextDiff::from_lines(original, replaced)
            .unified_diff()
            .header(&old_path, &new_path)
    );
}
