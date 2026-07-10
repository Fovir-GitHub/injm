use crate::core::types::Result;
use std::io::{self, Read};

pub fn read_stdin() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}
