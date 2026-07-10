use std::io::Write;
use std::process::Command;

fn injm_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_injm"))
}

#[test]
fn test_basic_injection() {
    let mut f = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(f, "fn main() {{").unwrap();
    writeln!(f, "    // injm begin").unwrap();
    writeln!(f, "    // injm end").unwrap();
    writeln!(f, "}}").unwrap();

    let _output = injm_bin()
        .arg("--output")
        .arg(f.path())
        .stdin(std::process::Stdio::piped())
        .output()
        .unwrap();

    let mut child = injm_bin()
        .arg("--output")
        .arg(f.path())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"    println!(\"hello\");")
        .unwrap();
    child.wait().unwrap();

    let result = std::fs::read_to_string(f.path()).unwrap();
    assert!(result.contains("println!(\"hello\")"));
    assert!(!result.contains("injm begin\n    // injm end"));
}

#[test]
fn test_dry_run() {
    let mut f = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(f, "fn main() {{").unwrap();
    writeln!(f, "    // injm begin").unwrap();
    writeln!(f, "    // injm end").unwrap();
    writeln!(f, "}}").unwrap();

    let original = std::fs::read_to_string(f.path()).unwrap();

    let mut child = injm_bin()
        .arg("--output")
        .arg(f.path())
        .arg("--dry-run")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"    println!(\"hello\");")
        .unwrap();
    child.wait().unwrap();

    let after = std::fs::read_to_string(f.path()).unwrap();
    assert_eq!(original, after);
}

#[test]
fn test_binary_file_returns_error() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(&[0x00, 0x01, 0x02, 0x03]).unwrap();

    let status = injm_bin()
        .arg("--output")
        .arg(f.path())
        .stdin(std::process::Stdio::piped())
        .status()
        .unwrap();

    assert!(!status.success());
}

#[test]
fn test_file_not_exist_returns_error() {
    let status = injm_bin()
        .arg("--output")
        .arg("not_exist.rs")
        .stdin(std::process::Stdio::piped())
        .status()
        .unwrap();

    assert!(!status.success());
}

#[test]
fn test_inject_with_id() {
    let mut f = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(f, "fn main() {{").unwrap();
    writeln!(f, "    // injm begin >greeting").unwrap();
    writeln!(f, "    // injm end").unwrap();
    writeln!(f, "    // injm begin >farewell").unwrap();
    writeln!(f, "    // injm end").unwrap();
    writeln!(f, "}}").unwrap();

    let mut child = injm_bin()
        .arg("--output")
        .arg(f.path())
        .arg("--id")
        .arg("greeting")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"    println!(\"hello\");")
        .unwrap();
    child.wait().unwrap();

    let result = std::fs::read_to_string(f.path()).unwrap();
    assert!(result.contains("println!(\"hello\")"));
    assert!(!result.contains("println!(\"bye\")"));
}

#[test]
fn test_inject_multiple_ids() {
    let mut f = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(f, "fn main() {{").unwrap();
    writeln!(f, "    // injm begin >first").unwrap();
    writeln!(f, "    old first").unwrap();
    writeln!(f, "    // injm end").unwrap();
    writeln!(f, "    // injm begin >second").unwrap();
    writeln!(f, "    old second").unwrap();
    writeln!(f, "    // injm end").unwrap();
    writeln!(f, "    // injm begin >third").unwrap();
    writeln!(f, "    old third").unwrap();
    writeln!(f, "    // injm end").unwrap();
    writeln!(f, "}}").unwrap();

    let mut child = injm_bin()
        .arg("--output")
        .arg(f.path())
        .arg("--id")
        .arg("first")
        .arg("--id")
        .arg("third")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"new content")
        .unwrap();
    child.wait().unwrap();

    let result = std::fs::read_to_string(f.path()).unwrap();
    assert!(!result.contains("old first"));
    assert!(result.contains("old second"));
    assert!(!result.contains("old third"));
    assert_eq!(result.matches("new content").count(), 2);
}

#[test]
fn test_inject_all_when_no_id_specified() {
    let mut f = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(f, "fn main() {{").unwrap();
    writeln!(f, "    // injm begin").unwrap();
    writeln!(f, "    old default").unwrap();
    writeln!(f, "    // injm end").unwrap();
    writeln!(f, "    // injm begin >first").unwrap();
    writeln!(f, "    old first").unwrap();
    writeln!(f, "    // injm end").unwrap();
    writeln!(f, "}}").unwrap();

    let mut child = injm_bin()
        .arg("--output")
        .arg(f.path())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"new content")
        .unwrap();
    child.wait().unwrap();

    let result = std::fs::read_to_string(f.path()).unwrap();
    assert!(!result.contains("old default"));
    assert!(result.contains("old first"));
}

#[test]
fn test_map_between_files() {
    let mut src = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(src, "fn main() {{").unwrap();
    writeln!(src, "    // injm begin <hello").unwrap();
    writeln!(src, "    println!(\"Hello injm\")").unwrap();
    writeln!(src, "    // injm end").unwrap();
    writeln!(src, "}}").unwrap();

    let mut dest = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(dest, "fn main() {{").unwrap();
    writeln!(dest, "    println!(\"Greeting from injm\")").unwrap();
    writeln!(dest, "    // injm begin >hello").unwrap();
    writeln!(dest, "    // injm end").unwrap();
    writeln!(dest, "}}").unwrap();

    let status = injm_bin()
        .arg("--input")
        .arg(src.path())
        .arg("--output")
        .arg(dest.path())
        .status()
        .unwrap();

    assert!(status.success());

    let result = std::fs::read_to_string(dest.path()).unwrap();
    assert!(result.contains("println!(\"Hello injm\")"));
    assert!(result.contains("println!(\"Greeting from injm\")"));
}

#[test]
fn test_map_missing_input_id_returns_error() {
    let mut src = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(src, "fn main() {{").unwrap();
    writeln!(src, "    // injm begin >hello").unwrap();
    writeln!(src, "    println!(\"Hello injm\")").unwrap();
    writeln!(src, "    // injm end").unwrap();
    writeln!(src, "}}").unwrap();

    let mut dest = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(dest, "fn main() {{").unwrap();
    writeln!(dest, "    // injm begin <not_exist").unwrap();
    writeln!(dest, "    // injm end").unwrap();
    writeln!(dest, "}}").unwrap();

    let status = injm_bin()
        .arg("--input")
        .arg(src.path())
        .arg("--output")
        .arg(dest.path())
        .status()
        .unwrap();

    assert!(!status.success());
}

#[test]
fn test_map_multiple_ids() {
    let mut src = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(src, "fn main() {{").unwrap();
    writeln!(src, "    // injm begin <first").unwrap();
    writeln!(src, "    println!(\"first\")").unwrap();
    writeln!(src, "    // injm end").unwrap();
    writeln!(src, "    // injm begin <second").unwrap();
    writeln!(src, "    println!(\"second\")").unwrap();
    writeln!(src, "    // injm end").unwrap();
    writeln!(src, "}}").unwrap();

    let mut dest = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(dest, "fn main() {{").unwrap();
    writeln!(dest, "    // injm begin >first").unwrap();
    writeln!(dest, "    // injm end").unwrap();
    writeln!(dest, "    // injm begin >second").unwrap();
    writeln!(dest, "    // injm end").unwrap();
    writeln!(dest, "}}").unwrap();

    let status = injm_bin()
        .arg("--input")
        .arg(src.path())
        .arg("--output")
        .arg(dest.path())
        .status()
        .unwrap();

    assert!(status.success());

    let result = std::fs::read_to_string(dest.path()).unwrap();
    assert!(result.contains("println!(\"first\")"));
    assert!(result.contains("println!(\"second\")"));
}

#[test]
fn test_map_dry_run() {
    let mut src = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(src, "fn main() {{").unwrap();
    writeln!(src, "    // injm begin <hello").unwrap();
    writeln!(src, "    println!(\"Hello injm\")").unwrap();
    writeln!(src, "    // injm end").unwrap();
    writeln!(src, "}}").unwrap();

    let mut dest = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(dest, "fn main() {{").unwrap();
    writeln!(dest, "    // injm begin >hello").unwrap();
    writeln!(dest, "    // injm end").unwrap();
    writeln!(dest, "}}").unwrap();

    let original = std::fs::read_to_string(dest.path()).unwrap();

    let status = injm_bin()
        .arg("--input")
        .arg(src.path())
        .arg("--output")
        .arg(dest.path())
        .arg("--dry-run")
        .status()
        .unwrap();

    assert!(status.success());

    let after = std::fs::read_to_string(dest.path()).unwrap();
    assert_eq!(original, after);
}
