use std::collections::TryReserveError;

use crate::overlap::overlap;
use crate::preprocess;

/// Builds a common superstring by repeatedly merging a maximum-overlap pair.
///
/// After [`preprocess`], each round considers all ordered pairs of distinct
/// current strings and merges the pair with the largest suffix-prefix overlap.
/// Ties choose the bytewise lexicographically smallest left string, then the
/// smallest right string. The merged string participates in later rounds.
///
/// The output contains every original input as a contiguous substring. Empty
/// input, or input consisting only of empty strings, returns an empty vector.
/// Choices are deterministic and independent of input order.
///
/// This heuristic can return a longer string than [`crate::exact`]. The
/// factor-two guarantee in OpenAI math family 128 applies to its separate
/// algorithm, which is not implemented here yet.
///
/// This reference implementation recomputes all pair overlaps every round:
/// `O(n^3)` overlap computations for `n` remaining inputs. Each overlap is found
/// by direct byte comparisons. It stores `O(L)` bytes of string content, where
/// `L` is the total remaining input length, and has no fixed input-count limit.
/// Preprocessing has additional costs.
///
/// # Errors
///
/// Returns [`TryReserveError`] if reserving storage for the working strings or
/// their contents fails, including when a merged string exceeds vector capacity
/// limits. Preprocessing uses ordinary infallible allocations.
///
/// ```
/// use superstring::{exact, greedy};
///
/// let inputs = [b"ab", b"ba", b"bb"];
/// assert_eq!(greedy(&inputs)?, b"ababb");
/// assert_eq!(exact(&inputs)?.len(), 4);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn greedy<S: AsRef<[u8]>>(strings: &[S]) -> Result<Vec<u8>, TryReserveError> {
    let reduced = preprocess(strings);
    let mut words = Vec::new();
    words.try_reserve_exact(reduced.len())?;
    for word in reduced {
        let mut owned = Vec::new();
        owned.try_reserve_exact(word.len())?;
        owned.extend_from_slice(word);
        words.push(owned);
    }

    while words.len() > 1 {
        // Sorted strings and ascending pair indices implement the documented
        // tie-breaking rule, including the all-zero-overlap case.
        let mut best = (0, 1, 0);
        for (left, a) in words.iter().enumerate() {
            for (right, b) in words.iter().enumerate() {
                if left == right {
                    continue;
                }
                let shared = overlap(a, b);
                if shared > best.2 {
                    best = (left, right, shared);
                }
            }
        }

        let (left, right, shared) = best;
        let right_word = words.remove(right);
        let mut merged = words.remove(left - usize::from(right < left));
        // try_reserve_exact checks the combined capacity before extending;
        // there is no unchecked addition of the two string lengths.
        merged.try_reserve_exact(right_word.len() - shared)?;
        merged.extend_from_slice(&right_word[shared..]);
        words.push(merged);
        words.sort_unstable();
    }

    Ok(words.pop().unwrap_or_default())
}
