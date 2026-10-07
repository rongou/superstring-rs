# Project guidance

## Scope

- Repository: `superstring-rs`. Intended Cargo package and library name: `superstring`.
- Build shortest common superstring algorithms in Rust, working toward a faithful
  implementation of OpenAI math family 128's polynomial-time 2-approximation.
- Start with a library over arbitrary byte strings. Measure length in bytes;
  do not assume UTF-8, normalize text, or use character indexing.
- The first milestone is preprocessing and an exact solver for small instances,
  followed by a greedy baseline and the paper's occurrence-count lower bound.

## Mathematical reference

- Upstream: https://github.com/openai/math
- Use upstream `main` as the working reference. Record the revision used when
  reporting proof verification or publishing benchmark results.
- Paper: `preprints/A-Polynomial-Time-2-Approximation-for-Shortest-Common-Superstring-September-24-2026/`.
  Read the TeX files in `build/sections/` when translating an algorithm step.
- Lean implementation: `lean/OAI/Computability/Superstring/`; the theorem entry
  point is `Main.lean`, and the executable solver is in `WalkOutput.lean`.
- A local reference checkout is available at `../math`. Use it for source
  inspection and keep Rust builds independent of that checkout.
- Lean is optional verification tooling, not a Rust build dependency. When
  verifying the upstream code, use its `lean-toolchain` and matching dependency
  pins, and target `OAI.Computability.Superstring.Main` rather than the whole
  collection.
- Cite the relevant paper section, lemma label, or Lean definition alongside
  nontrivial translated logic. Explain departures from the reference algorithm.
- Distinguish the paper's theorem, upstream proof checks actually performed, and
  evidence about this Rust implementation. Passing tests does not formally verify
  the Rust port. Do not attribute the factor-two guarantee to ordinary greedy.

## Implementation conventions

- Prefer safe Rust, a small API, and few dependencies. Establish a readable
  reference implementation before optimizing based on measurements.
- Remove empty strings, duplicates, and inputs contained in another input before
  algorithms that require a substring-free instance. Validate output coverage
  against the original input, including empty and duplicate entries.
- Make tie-breaking deterministic. Do not let hash-table iteration order decide
  algorithmic choices. Prefer stable IDs for interned words, vertices, and edges.
- Preserve edge multiplicities. Keep graph operations, periodic-layer operations,
  and budget accounting explicit enough to check their invariants independently.
- Periodic window coordinates can be negative. Use signed coordinates and
  Euclidean modulo for phases; guard conversions, count arithmetic, and size
  calculations against overflow.
- Document the exact solver's exponential resource cost and enforce a practical
  input limit before allocating its state table.

## Validation

- Use the small-instance exact solver as an oracle, validating it independently
  on tiny exhaustive cases. Compare solution lengths and coverage; different
  valid algorithms need not produce identical strings.
- Cover empty inputs, empty strings, duplicates, containment, zero overlaps,
  overlap ties, arbitrary bytes, repeated symbols, and competing periodic words.
- Use deterministic exhaustive or seeded generated cases for algorithm changes.
  For the paper's algorithm, check coverage and `output_len <= 2 * optimum` on
  small instances; check `W <= optimum` and `output_len <= 2 * W` when available.
- Check intermediate invariants where applicable: nonnegative edge counts,
  balanced degrees, preserved edge multisets, connected final support, and
  connection cost within the assigned budgets. Keep counterexamples as regressions.
- Once the crate exists, run these checks for Rust changes:

  ```sh
  cargo fmt --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-targets --all-features
  cargo test --doc
  ```

- For documentation-only changes, check the diff and referenced paths. Report
  checks actually run and any unresolved limitations.
