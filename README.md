# superstring-rs

`superstring` is a Rust library for shortest common superstrings over arbitrary
byte strings. It currently provides deterministic preprocessing, an exact
solver for small instances, a maximum-overlap greedy baseline, and the paper's
occurrence-count lower bound, as a foundation for implementing the polynomial-time
2-approximation in [OpenAI math family 128](https://github.com/openai/math/blob/main/lean/docs/128.md).
The approximation algorithm is not implemented yet.

```rust
use superstring::{exact, greedy, occurrence_counts, preprocess};

let inputs = [b"abc".as_slice(), b"bcd".as_slice(), b"bc".as_slice()];
assert_eq!(preprocess(&inputs), vec![b"abc".as_slice(), b"bcd"]);
assert_eq!(exact(&inputs).unwrap(), b"abcd");
assert_eq!(greedy(&inputs).unwrap(), b"abcd");
assert_eq!(occurrence_counts(&inputs).unwrap().lower_bound(), 4);
```

`preprocess` returns borrowed slices in bytewise lexicographic order, removing
empty strings, duplicates, and strings contained in another input. Both solvers
perform preprocessing automatically and return an owned `Vec<u8>` containing
every original input as a contiguous substring. Empty input produces an empty
output. None of the functions require valid UTF-8.

The exact solver uses subset dynamic programming over suffix-prefix overlaps.
With `n` strings remaining after preprocessing, the DP takes `O(n^2 * 2^n)` time
and `O(n * 2^n)` memory, in addition to preprocessing and overlap computation.
It accepts at most `MAX_EXACT_STRINGS = 18` remaining strings; the largest table
occupies 36 MiB on a 64-bit machine. Larger reduced instances return
`ExactError::TooManyStrings` before allocating that table. Equal-length solutions
are chosen deterministically, independent of input order, without promising the
lexicographically smallest answer.

The greedy solver repeatedly merges the ordered pair with the largest
suffix-prefix overlap. Ties choose the lexicographically smallest left string,
then right string. It has no fixed input-count limit, but this straightforward
implementation recomputes all overlaps each round (`O(n^3)` overlap computations).
It is a comparison baseline; the paper's factor-two guarantee applies to its
separate algorithm. For example, on `ab`, `ba`, `bb`, greedy produces `ababb`
(5 bytes), while an optimal answer is `abba` (4 bytes).

`occurrence_counts` implements Section 2's descending recursion over distinct
nonempty substrings, including its periodic blocking rules. It returns borrowed
substrings and forced counts `m(s)`, each bounded by the number of overlapping
occurrences in any common superstring. `lower_bound()` returns `W`, the sum of
the one-byte counts, which is at most the optimal superstring length. The bound
can be strict: on `ab`, `ba`, `bb`, it is 3 while the optimum is 4. `count(s)`
looks up an individual count; `entries()` iterates in bytewise lexicographic order.
Empty input gives an empty table and a zero bound. This initial implementation
enumerates substrings, periods, and possible blockers directly. Its runtime is
polynomial but can be expensive on long strings; it is intended for small inputs
while the remaining algorithm is developed.

Run a small comparison of lower bounds and solution lengths with:

```sh
cargo run --example compare
```

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
periodic strings, and the exact solver's input limit. Greedy tests check coverage,
determinism, and lengths against the exact solver, including a suboptimal case;
they do not assert a factor-two bound.

Count tests check the defining recurrence, `W <= optimum`, and counts against
actual overlapping occurrences in exact, greedy, and concatenated superstrings.
They also enumerate all short binary superstrings of tiny instances and cover
blocking multiplicities, periodic phases, and blockers shorter than the word
that triggers a rule. These tests provide evidence about the Rust implementation;
they do not formally verify it.

Next steps are the balanced base graph and the paper's layer and connection
algorithms. See [AGENTS.md](AGENTS.md) for reference
locations and implementation conventions.

## License

Licensed under either the [MIT license](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
