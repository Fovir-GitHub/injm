# Roadmap

## Vision

`injm` injects content into marked regions in source files.

## v0.1.0

- [x] Use `clap` to create the CLI interface
- [x] Detect programming language using `treesitter`
- [x] Extract comments from a giving file using `treesitter`
- [x] Replace content between comments `injm begin` and `injm end` with stdin input
- [x] Write the result back to the file using `-o` or `--output` flag

The usage looks like:

```bash
cat src.txt | injm -o dest.rs
```

## v0.2.0

- [x] Support `--dry-run` to preview changes without writing
- [x] Skip binary files
- [x] Error messages when markers are missing or mismatched

## v0.3.0

- [x] Support multiple marker pairs inside a file with an `ID` identifier
- [x] Allow the user to specify which block to insert

The usage looks like:

`dest.rs`

```rust
fn main() {
    // injm begin :first
    // injm end :first

    // injm begin :second
    // injm end :second
}
```

Then,

```bash
cat src.txt | injm -o dest.rs --id first
```

## v0.4.0

Synchronize between files.

- [x] Read source content from `--input` or `-i`
- [x] Use `<id` and `>id` to specify input or output
- [x] Report missing `<id` IDs

Example:

`src.rs`

```rust
fn main() {
    // injm begin <hello
    println!("Hello, world!")
    // injm end
}
```

`dest.rs`

```rust
fn main() {
    // injm begin >hello
    // injm end
}
```

Then,

```bash
injm -i src.rs -o dest.rs
```

And `dest.rs` becomes:

```rust
fn main() {
    // injm begin >hello
    println!("Hello, world!")
    // injm end
}
```

## v0.5.0

Synchronize directories.

- [ ] Scan supported source files recursively
- [ ] Accept directories as input and output
- [ ] Create `list` subcommand to list all marker regions

## v0.6.0

Improve preview and verification.

- [ ] Add check mode
- [ ] Improve error messages (Show line numbers of mismatched markers)
- [ ] Return non-zero exit code when synchronization is needed
- [ ] Show unified diff output

Example:

```bash
injm check -i src -o docs
```

Perfect for CI.

## v0.7.0

Improve project configuration.

- [ ] Support `injm.toml`
- [ ] Configure include/exclude patterns
- [ ] Configure marker or marker prefix
- [ ] Configure default source/output mappings

Example:

```toml
[input]
path = "examples"

[output]
path = "docs"

exclude = [
    "target/**",
    "vendor/**"
]
```

Then simply run:

```bash
injm
```
