use std::io::Write;
use std::process::Command;

fn injm_bin_list() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_injm"));
    cmd.arg("list");
    cmd
}

fn injm_bin_inject() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_injm"));
    cmd.arg("inject");
    cmd
}

fn inject_stdin(path: &std::path::Path, input: &[u8]) -> std::process::ExitStatus {
    let mut child = injm_bin_inject()
        .arg("--output")
        .arg(path)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.as_mut().unwrap().write_all(input);
    child.wait().unwrap()
}

#[test]
fn test_basic_injection() {
    let mut f = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(f, "fn main() {{").unwrap();
    writeln!(f, "    // injm begin").unwrap();
    writeln!(f, "    // injm end").unwrap();
    writeln!(f, "}}").unwrap();

    let _output = injm_bin_inject()
        .arg("--output")
        .arg(f.path())
        .stdin(std::process::Stdio::piped())
        .output()
        .unwrap();

    let mut child = injm_bin_inject()
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

    let mut child = injm_bin_inject()
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

    let status = injm_bin_inject()
        .arg("--output")
        .arg(f.path())
        .stdin(std::process::Stdio::piped())
        .status()
        .unwrap();

    assert!(!status.success());
}

#[test]
fn test_file_not_exist_returns_error() {
    let status = injm_bin_inject()
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

    let mut child = injm_bin_inject()
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

    let mut child = injm_bin_inject()
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

    let mut child = injm_bin_inject()
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

    let status = injm_bin_inject()
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
    writeln!(src, "    // injm begin <hello").unwrap();
    writeln!(src, "    println!(\"Hello injm\")").unwrap();
    writeln!(src, "    // injm end").unwrap();
    writeln!(src, "}}").unwrap();

    let mut dest = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(dest, "fn main() {{").unwrap();
    writeln!(dest, "    // injm begin >not_exist").unwrap();
    writeln!(dest, "    // injm end").unwrap();
    writeln!(dest, "}}").unwrap();

    let status = injm_bin_inject()
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

    let status = injm_bin_inject()
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

    let status = injm_bin_inject()
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

#[test]
fn test_unclosed_marker_returns_error() {
    let mut f = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(f, "fn main() {{").unwrap();
    writeln!(f, "    // injm begin").unwrap();
    writeln!(f, "}}").unwrap();

    let status = inject_stdin(f.path(), b"x");
    assert!(!status.success());
}

#[test]
fn test_map_input_file_not_exist_returns_error() {
    let mut dest = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(dest, "// injm begin >hello").unwrap();
    writeln!(dest, "// injm end").unwrap();

    let status = injm_bin_inject()
        .arg("--input")
        .arg("not_exist_input.rs")
        .arg("--output")
        .arg(dest.path())
        .status()
        .unwrap();

    assert!(!status.success());
}

#[test]
fn test_reinjection_replaces_content() {
    let mut f = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(f, "// injm begin").unwrap();
    writeln!(f, "// injm end").unwrap();

    assert!(inject_stdin(f.path(), b"first content").success());
    assert!(inject_stdin(f.path(), b"second content").success());

    let result = std::fs::read_to_string(f.path()).unwrap();
    assert!(result.contains("second content"));
    assert!(!result.contains("first content"));
    assert!(result.contains("// injm begin"));
    assert!(result.contains("// injm end"));
}

#[test]
fn test_glob_multiple_output_files() {
    let dir = tempfile::tempdir().unwrap();

    for name in ["a.rs", "b.rs"] {
        let path = dir.path().join(name);
        std::fs::write(&path, "// injm begin\n// injm end\n").unwrap();
    }

    let status = injm_bin_inject()
        .arg("--output")
        .arg(dir.path().join("*.rs"))
        .stdin(std::process::Stdio::piped())
        .status()
        .unwrap();

    assert!(status.success());

    for name in ["a.rs", "b.rs"] {
        let result = std::fs::read_to_string(dir.path().join(name)).unwrap();
        assert!(result.contains("// injm begin"));
    }
}

#[test]
fn test_glob_no_match_returns_error() {
    let dir = tempfile::tempdir().unwrap();

    let status = injm_bin_inject()
        .arg("--output")
        .arg(dir.path().join("*.rs"))
        .stdin(std::process::Stdio::piped())
        .status()
        .unwrap();

    assert!(!status.success());
}

#[test]
fn test_input_glob() {
    let dir = tempfile::tempdir().unwrap();

    std::fs::write(
        dir.path().join("a.rs"),
        r#"
// injm begin <hello
println!("hello");
// injm end
"#,
    )
    .unwrap();

    std::fs::write(
        dir.path().join("b.rs"),
        r#"
// injm begin <world
println!("world");
// injm end
"#,
    )
    .unwrap();

    let mut dest = tempfile::NamedTempFile::with_suffix(".rs").unwrap();

    writeln!(dest, "// injm begin >hello").unwrap();
    writeln!(dest, "// injm end").unwrap();
    writeln!(dest, "// injm begin >world").unwrap();
    writeln!(dest, "// injm end").unwrap();

    let status = injm_bin_inject()
        .arg("--input")
        .arg(dir.path().join("*.rs"))
        .arg("--output")
        .arg(dest.path())
        .status()
        .unwrap();

    assert!(status.success());

    let result = std::fs::read_to_string(dest.path()).unwrap();
    assert!(result.contains("hello"));
    assert!(result.contains("world"));
}

#[test]
fn test_output_glob() {
    let mut src = tempfile::NamedTempFile::with_suffix(".rs").unwrap();

    writeln!(src, "// injm begin <hello").unwrap();
    writeln!(src, "println!(\"hello\");").unwrap();
    writeln!(src, "// injm end").unwrap();

    let dir = tempfile::tempdir().unwrap();

    for name in ["a.rs", "b.rs"] {
        std::fs::write(dir.path().join(name), "// injm begin >hello\n// injm end\n").unwrap();
    }

    let status = injm_bin_inject()
        .arg("--input")
        .arg(src.path())
        .arg("--output")
        .arg(dir.path().join("*.rs"))
        .status()
        .unwrap();

    assert!(status.success());

    for name in ["a.rs", "b.rs"] {
        let result = std::fs::read_to_string(dir.path().join(name)).unwrap();
        assert!(result.contains("println!(\"hello\")"));
    }
}

#[test]
fn test_recursive_glob() {
    let dir = tempfile::tempdir().unwrap();

    std::fs::create_dir_all(dir.path().join("src/nested")).unwrap();

    std::fs::write(dir.path().join("a.rs"), "// injm begin\n// injm end\n").unwrap();

    std::fs::write(dir.path().join("b.txt"), "").unwrap();

    std::fs::write(dir.path().join("src/c.rs"), "// injm begin\n// injm end\n").unwrap();

    std::fs::write(
        dir.path().join("src/nested/d.rs"),
        "// injm begin\n// injm end\n",
    )
    .unwrap();

    let status = injm_bin_inject()
        .arg("--output")
        .arg(dir.path().join("**/*.rs"))
        .stdin(std::process::Stdio::piped())
        .status()
        .unwrap();

    assert!(status.success());
}

#[test]
fn test_duplicate_input_id_returns_error() {
    let dir = tempfile::tempdir().unwrap();

    std::fs::write(
        dir.path().join("a.rs"),
        r#"
// injm begin <hello
println!("a");
// injm end
"#,
    )
    .unwrap();

    std::fs::write(
        dir.path().join("b.rs"),
        r#"
// injm begin <hello
println!("b");
// injm end
"#,
    )
    .unwrap();

    let mut dest = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(dest, "// injm begin >hello").unwrap();
    writeln!(dest, "// injm end").unwrap();

    let status = injm_bin_inject()
        .arg("--input")
        .arg(dir.path().join("*.rs"))
        .arg("--output")
        .arg(dest.path())
        .status()
        .unwrap();

    assert!(!status.success());
}

#[test]
fn test_multiple_input_files() {
    let mut src1 = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(src1, "// injm begin <hello").unwrap();
    writeln!(src1, "println!(\"hello\");").unwrap();
    writeln!(src1, "// injm end").unwrap();

    let mut src2 = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(src2, "// injm begin <world").unwrap();
    writeln!(src2, "println!(\"world\");").unwrap();
    writeln!(src2, "// injm end").unwrap();

    let mut dest = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(dest, "// injm begin >hello").unwrap();
    writeln!(dest, "// injm end").unwrap();
    writeln!(dest, "// injm begin >world").unwrap();
    writeln!(dest, "// injm end").unwrap();

    let status = injm_bin_inject()
        .arg("--input")
        .arg(src1.path())
        .arg(src2.path())
        .arg("--output")
        .arg(dest.path())
        .status()
        .unwrap();

    assert!(status.success());

    let result = std::fs::read_to_string(dest.path()).unwrap();
    assert!(result.contains("println!(\"hello\")"));
    assert!(result.contains("println!(\"world\")"));
}

#[test]
fn test_multiple_output_files() {
    let mut src = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(src, "// injm begin <hello").unwrap();
    writeln!(src, "println!(\"hello\");").unwrap();
    writeln!(src, "// injm end").unwrap();

    let mut out1 = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(out1, "// injm begin >hello").unwrap();
    writeln!(out1, "// injm end").unwrap();

    let mut out2 = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(out2, "// injm begin >hello").unwrap();
    writeln!(out2, "// injm end").unwrap();

    let status = injm_bin_inject()
        .arg("--input")
        .arg(src.path())
        .arg("--output")
        .arg(out1.path())
        .arg(out2.path())
        .status()
        .unwrap();

    assert!(status.success());

    for file in [out1.path(), out2.path()] {
        let result = std::fs::read_to_string(file).unwrap();
        assert!(result.contains("println!(\"hello\")"));
    }
}

#[test]
fn test_multiple_input_globs() {
    let dir = tempfile::tempdir().unwrap();

    let dir1 = dir.path().join("a");
    let dir2 = dir.path().join("b");

    std::fs::create_dir(&dir1).unwrap();
    std::fs::create_dir(&dir2).unwrap();

    std::fs::write(
        dir1.join("hello.rs"),
        "// injm begin <hello\nprintln!(\"hello\");\n// injm end\n",
    )
    .unwrap();

    std::fs::write(
        dir2.join("world.rs"),
        "// injm begin <world\nprintln!(\"world\");\n// injm end\n",
    )
    .unwrap();

    let mut dest = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(dest, "// injm begin >hello").unwrap();
    writeln!(dest, "// injm end").unwrap();
    writeln!(dest, "// injm begin >world").unwrap();
    writeln!(dest, "// injm end").unwrap();

    let status = injm_bin_inject()
        .arg("--input")
        .arg(dir1.join("*.rs"))
        .arg(dir2.join("*.rs"))
        .arg("--output")
        .arg(dest.path())
        .status()
        .unwrap();

    assert!(status.success());

    let result = std::fs::read_to_string(dest.path()).unwrap();
    assert!(result.contains("println!(\"hello\")"));
    assert!(result.contains("println!(\"world\")"));
}

#[test]
fn test_multiple_output_globs() {
    let mut src = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
    writeln!(src, "// injm begin <hello").unwrap();
    writeln!(src, "println!(\"hello\");").unwrap();
    writeln!(src, "// injm end").unwrap();

    let dir = tempfile::tempdir().unwrap();

    let out1 = dir.path().join("a");
    let out2 = dir.path().join("b");

    std::fs::create_dir(&out1).unwrap();
    std::fs::create_dir(&out2).unwrap();

    for path in [out1.join("one.rs"), out2.join("two.rs")] {
        std::fs::write(&path, "// injm begin >hello\n// injm end\n").unwrap();
    }

    let status = injm_bin_inject()
        .arg("--input")
        .arg(src.path())
        .arg("--output")
        .arg(out1.join("*.rs"))
        .arg(out2.join("*.rs"))
        .status()
        .unwrap();

    assert!(status.success());

    for path in [out1.join("one.rs"), out2.join("two.rs")] {
        let result = std::fs::read_to_string(path).unwrap();
        assert!(result.contains("println!(\"hello\")"));
    }
}

#[test]
fn test_list_single_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rs");
    std::fs::write(
        &path,
        "// injm begin <input_id\ncontent\n// injm end\n// injm begin >output_id\ncontent\n// injm end\n",
    )
    .unwrap();

    let output = injm_bin_list().arg(&path).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("input_id"));
    assert!(stdout.contains("output_id"));
    assert!(stdout.contains("input "));
    assert!(stdout.contains("output"));
}

#[test]
fn test_list_json_format() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rs");
    std::fs::write(&path, "// injm begin <my_id\ncontent\n// injm end\n").unwrap();

    let output = injm_bin_list()
        .arg("--format")
        .arg("json")
        .arg(&path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""id": "my_id""#));
    assert!(stdout.contains(r#""marker_type": "input""#));
    assert!(stdout.contains(&format!(r#""file": "{}""#, path.to_string_lossy())));
}

#[test]
fn test_list_no_markers() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rs");
    std::fs::write(&path, "fn main() {}\n").unwrap();

    let output = injm_bin_list().arg(&path).output().unwrap();
    assert!(output.status.success());
}

#[test]
fn test_list_short_flag() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rs");
    std::fs::write(&path, "// injm begin <my_id\ncontent\n// injm end\n").unwrap();

    let output = injm_bin_list()
        .arg("-f")
        .arg("json")
        .arg(&path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""id": "my_id""#));
}

#[test]
fn test_list_binary_file_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rs");
    std::fs::write(&path, [0x00, 0x01, 0x02, 0x03]).unwrap();

    let status = injm_bin_list().arg(&path).status().unwrap();
    assert!(!status.success());
}

#[test]
fn test_list_file_not_exist_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let status = injm_bin_list()
        .arg(dir.path().join("not_exist.rs"))
        .status()
        .unwrap();
    assert!(!status.success());
}

#[test]
fn test_list_multiple_input_ids() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rs");
    std::fs::write(
        &path,
        "// injm begin <first <second\ncontent\n// injm end\n",
    )
    .unwrap();

    let output = injm_bin_list()
        .arg(&path)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();
    assert_eq!(rows.len(), 2);
    let ids: Vec<&str> = rows.iter().map(|r| r["id"].as_str().unwrap()).collect();
    assert!(ids.contains(&"first"));
    assert!(ids.contains(&"second"));
}

#[test]
fn test_list_input_and_output_same_block() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rs");
    std::fs::write(
        &path,
        "// injm begin <in_id >out_id\ncontent\n// injm end\n",
    )
    .unwrap();

    let output = injm_bin_list()
        .arg(&path)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();
    assert_eq!(rows.len(), 2);
    let types: Vec<&str> = rows
        .iter()
        .map(|r| r["marker_type"].as_str().unwrap())
        .collect();
    assert!(types.contains(&"input"));
    assert!(types.contains(&"output"));
}

#[test]
fn test_list_anonymous_block_produces_no_rows() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rs");
    std::fs::write(&path, "// injm begin\ncontent\n// injm end\n").unwrap();

    let output = injm_bin_list()
        .arg(&path)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();
    assert!(rows.is_empty());
}

#[test]
fn test_list_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.rs");
    std::fs::write(&path, "").unwrap();

    let output = injm_bin_list().arg(&path).output().unwrap();
    assert!(output.status.success());
}

#[test]
fn test_list_no_match_returns_error() {
    let dir = tempfile::tempdir().unwrap();

    let status = injm_bin_list()
        .arg(dir.path().join("*.rs"))
        .status()
        .unwrap();

    assert!(!status.success());
}

#[test]
fn test_list_multiple_patterns() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("a.rs"),
        "// injm begin <id_a\ncontent\n// injm end\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("b.rs"),
        "// injm begin <id_b\ncontent\n// injm end\n",
    )
    .unwrap();

    let output = injm_bin_list()
        .arg(dir.path().join("a.rs"))
        .arg(dir.path().join("b.rs"))
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("id_a"));
    assert!(stdout.contains("id_b"));
}

#[test]
fn test_list_directory_recursive() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("sub")).unwrap();
    std::fs::write(
        dir.path().join("a.rs"),
        "// injm begin <id_a\ncontent\n// injm end\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("sub/b.rs"),
        "// injm begin <id_b\ncontent\n// injm end\n",
    )
    .unwrap();

    let output = injm_bin_list()
        .arg(dir.path().to_string_lossy().as_ref())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("id_a"));
    assert!(stdout.contains("id_b"));
}
