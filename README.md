# superstring-rs

`superstring` is a Rust library for shortest common superstrings over arbitrary
byte strings. It currently provides deterministic preprocessing and an exact
solver for small instances, as a foundation for implementing the polynomial-time
2-approximation in [OpenAI math family 128](https://github.com/openai/math/blob/main/lean/docs/128.md).
The approximation algorithm is not implemented yet.

```rust
use superstring::{exact, preprocess};

let inputs = [b"abc".as_slice(), b"bcd".as_slice(), b"bc".as_slice()];
assert_eq!(preprocess(&inputs), vec![b"abc".as_slice(), b"bcd"]);
assert_eq!(exact(&inputs).unwrap(), b"abcd");
```

`preprocess` returns borrowed slices in bytewise lexicographic order, removing
empty strings, duplicates, and strings contained in another input. `exact`
performs preprocessing automatically and returns an owned `Vec<u8>` containing
every original input as a contiguous substring. Empty input produces an empty
output. Neither function requires valid UTF-8.

The exact solver uses subset dynamic programming over suffix-prefix overlaps.
With `n` strings remaining after preprocessing, the DP takes `O(n^2 * 2^n)` time
and `O(n * 2^n)` memory, in addition to preprocessing and overlap computation.
It accepts at most `MAX_EXACT_STRINGS = 18` remaining strings; the largest table
occupies 36 MiB on a 64-bit machine. Larger reduced instances return
`ExactError::TooManyStrings` before allocating that table. Equal-length solutions
are chosen deterministically, independent of input order, without promising the
lexicographically smallest answer.

## Development

The crate has no external dependencies. Rust builds and tests do not require Lean
or a checkout of the math repository.

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo test --doc
```

Tests check coverage against the original inputs and optimal lengths against
independent literal-string enumeration on tiny cases and permutation search on
larger cases. They include arbitrary bytes, redundant inputs, overlap ties,
periodic strings, and the exact solver's input limit.

Next steps are a greedy baseline, the paper's occurrence-count lower bound, and
its layer and connection algorithms. See [AGENTS.md](AGENTS.md) for reference
locations and implementation conventions.

## License

Licensed under either the [MIT license](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
