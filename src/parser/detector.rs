use super::{ParserError, Result};
use std::fs;

pub(crate) fn detect(path: &str) -> Result<&'static str> {
    // Detect language from file path or extension.
    if let Some(lang) = tree_sitter_language_pack::detect_language(path) {
        return Ok(lang);
    }

    // Detect from shebang line.
    if let Ok(content) = fs::read_to_string(path)
        && let Some(lang) = tree_sitter_language_pack::detect_language_from_content(&content)
    {
        return Ok(lang);
    }

    Err(ParserError::UnsupportedFileType {
        path: path.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_from_extension() {
        assert_eq!(detect("main.rs").unwrap(), "rust");
        assert_eq!(detect("main.py").unwrap(), "python");
        assert_eq!(detect("main.js").unwrap(), "javascript");
        assert_eq!(detect("main.go").unwrap(), "go");
        assert_eq!(detect("main.md").unwrap(), "markdown");
    }

    #[test]
    fn test_detect_unknown_extension() {
        assert!(detect("main.xyz").is_err());
        assert!(detect("noextension").is_err());
    }

    #[test]
    fn test_detect_from_shebang() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "#!/usr/bin/env python3").unwrap();
        writeln!(f, "print('hello')").unwrap();
        assert_eq!(detect(&f.path().to_string_lossy()).unwrap(), "python");

        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "#!/usr/bin/env bash").unwrap();
        writeln!(f, "echo hello").unwrap();
        assert_eq!(detect(&f.path().to_string_lossy()).unwrap(), "bash");
    }
}
