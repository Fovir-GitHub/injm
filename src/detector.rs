use std::{fs, path::Path};

pub fn detect(path: &Path) -> Option<&'static str> {
    // Detect language from file path or extension.
    if let Some(lang) = tree_sitter_language_pack::detect_language(path.to_str()?) {
        return Some(lang);
    }

    // Detect from shebang line.
    if let Ok(content) = fs::read_to_string(path) {
        if let Some(lang) = tree_sitter_language_pack::detect_language_from_content(&content) {
            return Some(lang);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_from_extension() {
        assert_eq!(detect(Path::new("main.rs")), Some("rust"));
        assert_eq!(detect(Path::new("main.py")), Some("python"));
        assert_eq!(detect(Path::new("main.js")), Some("javascript"));
        assert_eq!(detect(Path::new("main.go")), Some("go"));
        assert_eq!(detect(Path::new("main.md")), Some("markdown"));
    }

    #[test]
    fn test_detect_unknown_extension() {
        assert_eq!(detect(Path::new("main.xyz")), None);
        assert_eq!(detect(Path::new("noextension")), None);
    }

    #[test]
    fn test_detect_from_shebang() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "#!/usr/bin/env python3").unwrap();
        writeln!(f, "print('hello')").unwrap();
        assert_eq!(detect(f.path()), Some("python"));

        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "#!/usr/bin/env bash").unwrap();
        writeln!(f, "echo hello").unwrap();
        assert_eq!(detect(f.path()), Some("bash"));
    }
}
