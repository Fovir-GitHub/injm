# injm

A CLI tool that injects content into marked regions in source files.

## Table of Contents

<!-- toc -->

- [Installation](#installation)
  - [Cargo](#cargo)
  - [Nix](#nix)
  - [Download Binary](#download-binary)
- [Usage](#usage)
  - [Basic Injection](#basic-injection)
  - [Inject into a Specific Region](#inject-into-a-specific-region)
  - [Sync Between Files](#sync-between-files)
  - [Multiple Files and Globs](#multiple-files-and-globs)
  - [List Markers](#list-markers)
  - [Dry Run](#dry-run)
- [Supported Languages](#supported-languages)
- [Roadmap](#roadmap)
- [License](#license)
- [Acknowledgement](#acknowledgement)

<!-- tocstop -->

## Installation

### Cargo

```bash
cargo install injm
```

### Nix

```bash
nix profile install github:Fovir-GitHub/injm
```

### Download Binary

Download the latest binary for your platform from [GitHub Releases](https://github.com/Fovir-GitHub/injm/releases/latest).

## Usage

`injm` uses subcommands. The main one is `inject`:

### Basic Injection

Mark a region in your source file with `injm begin` and `injm end` comments:

`dest.rs`

```rust
fn main() {
    // injm begin
    // injm end
}
```

Then pipe content into `injm`:

```bash
echo -n 'println!("Hello, world!")' | injm inject --output dest.rs
```

Result:

`dest.rs`

```rust
fn main() {
    // injm begin
println!("Hello, world!");
    // injm end
}
```

Running `injm inject` again will replace the content between the markers:

```bash
cat src.txt | injm inject --output dest.rs
```

### Inject into a Specific Region

Give a region an output ID with `>id`, then target it with `--id`:

`dest.rs`

```rust
fn main() {
    // injm begin >greeting
    // injm end

    // injm begin >farewell
    // injm end
}
```

Inject into a specific region:

```bash
echo -n 'println!("Hello!")' | injm inject --output dest.rs --id greeting
```

Inject into multiple regions at once:

```bash
echo -n 'println!("Hello!")' | injm inject --output dest.rs --id greeting --id farewell
```

If `--id` is not specified, only regions **without** an ID are injected; regions with a `>id` are left untouched.

### Sync Between Files

Instead of piping from stdin, copy content between files with `--input`.
Mark the source region with `<id` (the content to read) and the destination
region with `>id` (where it goes):

`src.rs`

```rust
fn main() {
    // injm begin <hello
    println!("Hello, world!");
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

Then sync:

```bash
injm inject --input src.rs --output dest.rs
```

`dest.rs` becomes:

```rust
fn main() {
    // injm begin >hello
    println!("Hello, world!");
    // injm end
}
```

A region may read from several sources by listing multiple `<id` markers.
If a `>id` in the output has no matching `<id` in the input, `injm` reports
the missing ID and exits with an error.

### Multiple Files and Globs

`--input` and `--output` accept multiple values and glob patterns:

```bash
# Multiple explicit files
injm inject --input src1.rs src2.rs --output dest.rs

# Sync to multiple outputs
injm inject --input src.rs --output out1.rs out2.rs

# Glob patterns
injm inject --input "src/**/*.rs" --output "docs/"

# Multiple globs
injm inject --input "mod_a/**/*.rs" "mod_b/**/*.rs" --output dest.rs
```

### List Markers

Preview all marker regions across files:

```bash
injm list src/
```

Output:

```
+-------------+----------+--------+-------+
| File        | ID       | Type   | Lines |
+-------------+----------+--------+-------+
| src/main.rs | hello    | output | 6-7   |
+-------------+----------+--------+-------+
| src/main.rs | hello    | input  | 23-24 |
+-------------+----------+--------+-------+
| src/cli.rs  | greeting | input  | 1-2   |
+-------------+----------+--------+-------+
```

JSON output:

```bash
injm list src/ --format json
```

Accepts positional arguments (files, globs, or directories). Falls back to
current directory when no argument is given.

### Dry Run

Preview the result without writing to the file:

```bash
cat src.txt | injm inject --output dest.rs --dry-run
```

## Supported Languages

`injm` uses [tree-sitter](https://tree-sitter.github.io/tree-sitter/) to parse source files, so markers are detected from actual comment nodes — not from string literals or other non-comment content.

Supports any language recognized by [tree-sitter-language-pack](https://github.com/kreuzberg-dev/tree-sitter-language-pack), including:

- Rust, C, C++
- Python, Ruby
- JavaScript, TypeScript
- Go, Java
- And [300+ more](https://github.com/kreuzberg-dev/tree-sitter-language-pack)

## Roadmap

See [ROADMAP.md](ROADMAP.md).

## License

MIT

## Acknowledgement

- [clap-rs/clap](https://github.com/clap-rs/clap): A full featured, fast Command Line Argument Parser for Rust.
- [rust-lang/glob](https://github.com/rust-lang/glob): Support for matching file paths against Unix shell style patterns.
- [serde-rs/serde](https://github.com/serde-rs/serde): Serialization framework for Rust.
- [xberg-io/tree-sitter-language-pack](https://github.com/xberg-io/tree-sitter-language-pack): Comprehensive tree-sitter grammar compilation with polyglot bindings — Rust, Python, Node.js, Go, Java, Ruby, Elixir, PHP, C#, WASM, Dart, Kotlin-Android, Swift, Zig, and CLI. 306+ languages.
- [zhiburt/tabled](https://github.com/zhiburt/tabled): An easy to use library for pretty print tables of Rust structs and enums.
