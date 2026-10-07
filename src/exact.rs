use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;

use crate::overlap::overlap;
use crate::preprocess;

/// Maximum number of strings supported by [`exact`] after preprocessing.
///
/// At 18 strings the dynamic-programming table occupies 36 MiB on a 64-bit
/// machine. The input, overlap table, and output require additional memory.
pub const MAX_EXACT_STRINGS: usize = 18;

/// An exact solution could not be computed within the solver's resource limits.
#[derive(Debug)]
pub enum ExactError {
    /// Too many strings remain after removing empty and redundant inputs.
    TooManyStrings {
        /// Number of strings remaining after preprocessing.
        count: usize,
        /// Maximum supported number of remaining strings.
        limit: usize,
    },
    /// An input-length sum or state-table size cannot be represented by `usize`.
    LengthOverflow,
    /// Memory for the dynamic-programming table or output could not be reserved.
    AllocationFailed(TryReserveError),
}

impl fmt::Display for ExactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyStrings { count, limit } => write!(
                f,
                "exact solver supports at most {limit} strings after preprocessing; got {count}"
            ),
            Self::LengthOverflow => {
                write!(
                    f,
                    "input lengths or state-table size exceed addressable limits"
                )
            }
            Self::AllocationFailed(error) => write!(f, "could not allocate solver memory: {error}"),
        }
    }
}

impl Error for ExactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AllocationFailed(error) => Some(error),
            _ => None,
        }
    }
}

/// Finds a shortest byte string containing every input as a contiguous substring.
///
/// Empty input, or input consisting only of empty strings, returns an empty
/// vector. The result is deterministic even if the inputs are reordered, but is
/// not necessarily the lexicographically smallest optimal superstring.
///
/// After [`preprocess`], with `n` remaining strings, the subset dynamic program
/// takes `O(n^2 * 2^n)` time and `O(n * 2^n)` space. Preprocessing and pairwise
/// overlap computation have additional costs. The limit [`MAX_EXACT_STRINGS`]
/// applies to the number of remaining strings, so larger collections of redundant
/// inputs can still be solved.
///
/// This is an exact small-instance oracle, separate from the planned
/// polynomial-time approximation algorithm.
///
/// # Errors
///
/// Returns [`ExactError::TooManyStrings`] if the reduced input exceeds the limit,
/// [`ExactError::LengthOverflow`] if size arithmetic overflows, or
/// [`ExactError::AllocationFailed`] if the DP table or output cannot be reserved.
pub fn exact<S: AsRef<[u8]>>(strings: &[S]) -> Result<Vec<u8>, ExactError> {
    let words = preprocess(strings);
    let n = words.len();
    if n > MAX_EXACT_STRINGS {
        return Err(ExactError::TooManyStrings {
            count: n,
            limit: MAX_EXACT_STRINGS,
        });
    }
    if n == 0 {
        return Ok(Vec::new());
    }

    // Every path appends at most the total input length. Checking this sum and
    // reserving a larger sentinel makes the DP's additions safe and unambiguous.
    let total_len = words
        .iter()
        .try_fold(0usize, |total, word| total.checked_add(word.len()))
        .ok_or(ExactError::LengthOverflow)?;
    let unreachable = total_len.checked_add(1).ok_or(ExactError::LengthOverflow)?;
    let state_count = 1usize
        .checked_shl(n as u32)
        .ok_or(ExactError::LengthOverflow)?;
    let cells = state_count
        .checked_mul(n)
        .ok_or(ExactError::LengthOverflow)?;

    let mut overlaps = vec![0; n * n];
    for (left, a) in words.iter().enumerate() {
        for (right, b) in words.iter().enumerate() {
            if left != right {
                overlaps[left * n + right] = overlap(a, b);
            }
        }
    }

    let mut lengths = Vec::new();
    lengths
        .try_reserve_exact(cells)
        .map_err(ExactError::AllocationFailed)?;
    lengths.resize(cells, unreachable);
    for (last, word) in words.iter().enumerate() {
        lengths[(1 << last) * n + last] = word.len();
    }

    // Once contained strings are removed, input occurrences in an optimum can
    // be ordered by their starts (and their ends). Merging that order with
    // maximal pairwise overlaps gives an optimum, so it suffices to find a
    // minimum-length Hamiltonian path in the complete overlap graph.
    // lengths[mask, last] stores its optimum for this subset and final string.
    for mask in 1..state_count {
        for last in 0..n {
            let current = lengths[mask * n + last];
            if current == unreachable {
                continue;
            }
            for (next, word) in words.iter().enumerate() {
                if mask & (1 << next) != 0 {
                    continue;
                }
                let next_mask = mask | (1 << next);
                let candidate = current + (word.len() - overlaps[last * n + next]);
                let best = &mut lengths[next_mask * n + next];
                *best = (*best).min(candidate);
            }
        }
    }

    let full = state_count - 1;
    let mut last = (0..n)
        .min_by_key(|&last| lengths[full * n + last])
        .expect("the reduced instance is nonempty");
    let output_len = lengths[full * n + last];

    // Recover predecessors from the length table to avoid a second exponential
    // table. On ties, use the first predecessor in canonical input order.
    let mut order = Vec::with_capacity(n);
    let mut mask = full;
    loop {
        order.push(last);
        let previous_mask = mask ^ (1 << last);
        if previous_mask == 0 {
            break;
        }
        last = (0..n)
            .find(|&previous| {
                if previous_mask & (1 << previous) == 0 {
                    return false;
                }
                let cost = lengths[previous_mask * n + previous];
                cost != unreachable
                    && cost + (words[last].len() - overlaps[previous * n + last])
                        == lengths[mask * n + last]
            })
            .expect("every nonsingleton reachable state has an optimal predecessor");
        mask = previous_mask;
    }
    order.reverse();
    drop(lengths);

    let mut output = Vec::new();
    output
        .try_reserve_exact(output_len)
        .map_err(ExactError::AllocationFailed)?;
    output.extend_from_slice(words[order[0]]);
    for pair in order.windows(2) {
        let (previous, next) = (pair[0], pair[1]);
        output.extend_from_slice(&words[next][overlaps[previous * n + next]..]);
    }
    debug_assert_eq!(output.len(), output_len);
    Ok(output)
}
