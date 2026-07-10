use crate::core::types::Result;
use std::{fs, path::Path};

pub fn detect(path: &Path) -> Result<&'static str> {
    // Detect language from file path or extension.
    if let Some(lang) =
        tree_sitter_language_pack::detect_language(path.to_str().ok_or("invalid path")?)
    {
        return Ok(lang);
    }

    // Detect from shebang line.
    if let Ok(content) = fs::read_to_string(path) {
        if let Some(lang) = tree_sitter_language_pack::detect_language_from_content(&content) {
            return Ok(lang);
        }
    }

    Err("unsupported file type".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_from_extension() {
        assert_eq!(detect(Path::new("main.rs")).unwrap(), "rust");
        assert_eq!(detect(Path::new("main.py")).unwrap(), "python");
        assert_eq!(detect(Path::new("main.js")).unwrap(), "javascript");
        assert_eq!(detect(Path::new("main.go")).unwrap(), "go");
        assert_eq!(detect(Path::new("main.md")).unwrap(), "markdown");
    }

    #[test]
    fn test_detect_unknown_extension() {
        assert!(detect(Path::new("main.xyz")).is_err());
        assert!(detect(Path::new("noextension")).is_err());
    }

    #[test]
    fn test_detect_from_shebang() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "#!/usr/bin/env python3").unwrap();
        writeln!(f, "print('hello')").unwrap();
        assert_eq!(detect(f.path()).unwrap(), "python");

        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "#!/usr/bin/env bash").unwrap();
        writeln!(f, "echo hello").unwrap();
        assert_eq!(detect(f.path()).unwrap(), "bash");
    }
}
