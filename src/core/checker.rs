use crate::core::types::{BlockRole, MarkerBlock, ParsedFile, Result};
use std::collections::HashSet;
use std::fs;

pub(crate) fn check_file(path: &str) -> Result<()> {
    if !fs::exists(path)? {
        return Err("file does not exist".into());
    }

    if is_binary_file(path)? {
        return Err("binary file".into());
    }

    Ok(())
}

pub fn check_missing_ids(output_files: &[ParsedFile], input_files: &[ParsedFile]) -> Result<()> {
    let input_blocks: HashSet<&String> = input_files
        .iter()
        .flat_map(|file| file.blocks.iter())
        .filter_map(|b| match &b.role {
            BlockRole::Input { ids, .. } => Some(ids.iter()),
            _ => None,
        })
        .flatten()
        .collect();

    if let Some(id) = output_files
        .iter()
        .flat_map(|file| file.blocks.iter())
        .filter_map(|b| match &b.role {
            BlockRole::Output { id } => id.as_ref(),
            _ => None,
        })
        .find(|&id| !input_blocks.contains(id))
    {
        return Err(format!("missing input id `{id}`").into());
    }

    Ok(())
}

pub fn check_duplicated_input_ids(blocks: &[MarkerBlock]) -> Result<()> {
    let mut seen = HashSet::new();

    for block in blocks {
        if let BlockRole::Input { ids, .. } = &block.role {
            for id in ids {
                if !seen.insert(id) {
                    return Err(format!("duplicated input id `{id}`").into());
                }
            }
        }
    }

    Ok(())
}

fn is_binary_file(path: &str) -> Result<bool> {
    use std::io::Read;
    let mut f = fs::File::open(path)?;
    let mut buffer = [0u8; 8192];
    let n = f.read(&mut buffer)?;
    Ok(buffer[..n].contains(&0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_file_not_exist() {
        assert!(check_file("not_exist.rs").is_err());
    }

    #[test]
    fn test_text_file() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "fn main() {{}}").unwrap();
        assert!(check_file(&f.path().to_string_lossy()).is_ok());
    }

    #[test]
    fn test_binary_file() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(&[0x00, 0x01, 0x02, 0x03]).unwrap();
        assert!(check_file(&f.path().to_string_lossy()).is_err());
    }

    #[test]
    fn test_empty_file() {
        let f = tempfile::NamedTempFile::new().unwrap();
        assert!(check_file(&f.path().to_string_lossy()).is_ok());
    }
}
