# Fail-safe file sequence

Implemented in Rust.
Inspired by this [Java implementation](https://commons.apache.org/proper/commons-transaction/apidocs/org/apache/commons/transaction/file/FileSequence.html)

![Crates.io](https://img.shields.io/crates/v/file-seq) ![GitHub Workflow Status](https://img.shields.io/github/workflow/status/jonhkr/rust-file-seq/Rust)
## Usage

```rust
let initial_value = 1;
let seq = FileSeq::new(store_dir, initial_value).unwrap();

// Get current value
seq.value().unwrap();

// Increment by 1 and get
seq.increment_and_get(1).unwrap();

// Get, then increment by 1
seq.get_and_increment(1).unwrap();
```

## Changelog

### 0.2.0 (2020-09-07)
- Ignore errors on `FileSeq::delete` function [\#1](https://github.com/jonhkr/rust-file-seq/pull/1)
