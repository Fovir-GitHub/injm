use clap::ValueEnum;
use serde::Serialize;
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
