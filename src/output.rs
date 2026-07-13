use serde::Serialize;
use tabled::{Table, Tabled};

use crate::cli::OutputFormat;
use crate::core::types::Result;

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
