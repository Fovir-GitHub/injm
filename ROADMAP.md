# Roadmap

## Vision

`injm` injects content into marked regions in source files.

## v0.1.0

- [ ] Use `clap` to create the CLI interface
- [ ] Detect programming language using `treesitter`
- [ ] Extract comments from a giving file using `treesitter`
- [ ] Replace content between comments `injm begin` and `injm end` with stdin input
- [ ] Write the result back to the file using `-o` or `--output` flag

The usage looks like:

```bash
cat src.txt | injm -o dest.rs
```

## v0.2.0

- [ ] Support `--dry-run` to preview changes without writing
- [ ] Skip binary files
- [ ] Error messages when markers are missing or mismatched

## v0.3.0

- [ ] Support multiple marker pairs inside a file with an `ID` identifier
- [ ] Allow the user to specify which block to insert

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

- [ ] Support "map" operation between two files by ID
- [ ] Provide `-i` and `--input` to specify the input file

Usage:

`src.rs`

```rust
fn main() {
    // injm begin :hello
    println!("Hello injm")
    // injm end :hello
}
```

`dest.rs`

```rust
fn main() {
    println!("Greeting from injm")
    // injm begin :hello
    // injm end :hello
}
```

Then, run

```bash
injm -i src.rs -o dest.rs
```

And `dest.rs` becomes

```rust
fn main() {
    println!("Greeting from injm")
    // injm begin :hello
    println!("Hello injm")
    // injm end :hello
}
```
