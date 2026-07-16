use crate::validator::ValidatorError;

use super::Result;
use std::{fs, path::Path};

pub(crate) fn validate_file(path: &Path) -> Result<()> {
    if !fs::exists(path)? {
        return Err(ValidatorError::FileNotExist {
            path: path.to_owned(),
        });
    }

    if is_binary_file(path)? {
        return Err(ValidatorError::BinaryFile {
            path: path.to_owned(),
        });
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
        assert!(validate_file(Path::new("not_exist.rs")).is_err());
    }

    #[test]
    fn test_text_file() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "fn main() {{}}").unwrap();
        assert!(validate_file(f.path()).is_ok());
    }

    #[test]
    fn test_binary_file() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(&[0x00, 0x01, 0x02, 0x03]).unwrap();
        assert!(validate_file(f.path()).is_err());
    }

    #[test]
    fn test_empty_file() {
        let f = tempfile::NamedTempFile::new().unwrap();
        assert!(validate_file(f.path()).is_ok());
    }
}
