# injm

A CLI tool that injects content into marked regions in source files.

## Table of Contents

<!-- toc -->

- [Installation](#installation)
  - [Cargo](#cargo)
  - [Nix](#nix)
  - [Download Binary](#download-binary)
- [Usage](#usage)
  - [Inject into a Specific Region](#inject-into-a-specific-region)
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
echo -n 'println!("Hello, world!")' | injm --output dest.rs
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

Running `injm` again will replace the content between the markers:

```bash
cat src.txt | injm --output dest.rs
```

### Inject into a Specific Region

Use `:id` to name a region:

`dest.rs`

```rust
fn main() {
    // injm begin :greeting
    // injm end :greeting

    // injm begin :farewell
    // injm end :farewell
}
```

Inject into a specific region with `--id`:

```bash
echo -n 'println!("Hello!")' | injm --output dest.rs --id greeting
```

Inject into multiple regions at once:

```bash
echo -n 'println!("Hello!")' | injm --output dest.rs --id greeting --id farewell
```

If `--id` is not specified, all regions without an ID will be injected.

### Dry Run

Preview the result without writing to the file:

```bash
cat src.txt | injm --output dest.rs --dry-run
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

- [clap-rs/clap](https://github.com/clap-rs/clap): A full featured, fast Command Line Argument Parser for Rust
- [xberg-io/tree-sitter-language-pack](https://github.com/xberg-io/tree-sitter-language-pack): Comprehensive tree-sitter grammar compilation with polyglot bindings — Rust, Python, Node.js, Go, Java, Ruby, Elixir, PHP, C#, WASM, Dart, Kotlin-Android, Swift, Zig, and CLI. 306+ languages.
