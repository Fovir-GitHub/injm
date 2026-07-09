use crate::types::Result;
use std::{fs, path::Path};

pub fn check_file(path: &Path) -> Result<()> {
    if !fs::exists(path)? {
        return Err("file does not exist".into());
    }

    if is_binary_file(path)? {
        return Err("binary file".into());
    }

    Ok(())
}

fn is_binary_file(path: &Path) -> Result<bool> {
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
        assert!(check_file(Path::new("not_exist.rs")).is_err());
    }

    #[test]
    fn test_text_file() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "fn main() {{}}").unwrap();
        assert!(check_file(f.path()).is_ok());
    }

    #[test]
    fn test_binary_file() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(&[0x00, 0x01, 0x02, 0x03]).unwrap();
        assert!(check_file(f.path()).is_err());
    }

    #[test]
    fn test_empty_file() {
        let f = tempfile::NamedTempFile::new().unwrap();
        assert!(check_file(f.path()).is_ok());
    }
}
