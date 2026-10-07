use std::cmp::Reverse;
use std::collections::{BTreeSet, TryReserveError};
use std::error::Error;
use std::fmt;

use crate::preprocess;

/// Forced occurrence counts for the distinct nonempty substrings of an instance.
///
/// For every common superstring `T`, each count `m(s)` is at most the number of
/// occurrences of `s` in `T`, including overlapping occurrences. The sum of the
/// one-byte counts is a lower bound on the shortest common superstring length.
/// These are the counts in Section 2 (`lem:counts`) of the family 128 paper.
///
/// Substring contents are borrowed from the inputs; no bytes are copied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OccurrenceCounts<'a> {
    words: Vec<&'a [u8]>,
    counts: Vec<usize>,
    lower_bound: usize,
}

impl<'a> OccurrenceCounts<'a> {
    /// Returns the length lower bound `W`, the sum of the one-byte counts.
    pub fn lower_bound(&self) -> usize {
        self.lower_bound
    }

    /// Returns the forced count of `word`.
    ///
    /// Returns zero for a word absent from the inputs' substrings. The empty
    /// word is unassigned and also returns zero.
    pub fn count(&self, word: &[u8]) -> usize {
        self.words
            .binary_search(&word)
            .map_or(0, |id| self.counts[id])
    }

    /// Iterates over all nonempty substrings and their counts in bytewise
    /// lexicographic order.
    pub fn entries(&self) -> impl ExactSizeIterator<Item = (&'a [u8], usize)> + '_ {
        self.words.iter().copied().zip(self.counts.iter().copied())
    }
}

/// Forced occurrence counts could not be computed within addressable limits.
#[derive(Debug)]
pub enum CountsError {
    /// An input-length sum, signed position, or count cannot be represented.
    ArithmeticOverflow,
    /// Memory for a count table or periodic rule list could not be reserved.
    AllocationFailed(TryReserveError),
}

impl fmt::Display for CountsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArithmeticOverflow => {
                write!(f, "occurrence-count arithmetic exceeds addressable limits")
            }
            Self::AllocationFailed(error) => write!(f, "could not allocate count memory: {error}"),
        }
    }
}

impl Error for CountsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AllocationFailed(error) => Some(error),
            _ => None,
        }
    }
}

impl From<TryReserveError> for CountsError {
    fn from(error: TryReserveError) -> Self {
        Self::AllocationFailed(error)
    }
}

/// Computes the paper's forced occurrence counts and length lower bound `W`.
///
/// Applies [`preprocess`] and then the descending-length recursion in Section 2
/// of the family 128 paper (`eq:extension-rules`, `eq:period-rule`, and `eq:W`).
/// Empty inputs give an empty table and a zero lower bound. Results do not depend
/// on input order. The lower bound need not equal the optimal length.
///
/// This is a straightforward polynomial-time reference implementation intended
/// for small instances. It enumerates all distinct substrings (up to `O(L^2)` for
/// total reduced input length `L`), all admissible periods, and possible blocking
/// placements. The repeated blocker scans can be expensive even for a few long
/// strings. This computes the first stage of the paper's algorithm, not a
/// superstring or the complete 2-approximation.
///
/// # Errors
///
/// Returns [`CountsError::ArithmeticOverflow`] if checked arithmetic overflows,
/// or [`CountsError::AllocationFailed`] if count tables or rule lists cannot be
/// reserved. Preprocessing and substring-set construction use standard Rust
/// allocations.
///
/// ```
/// use superstring::occurrence_counts;
///
/// let inputs = [b"aaaaa"];
/// let counts = occurrence_counts(&inputs)?;
/// assert_eq!(counts.count(b"aaa"), 3);
/// assert_eq!(counts.lower_bound(), 5);
/// # Ok::<(), superstring::CountsError>(())
/// ```
pub fn occurrence_counts<S: AsRef<[u8]>>(
    strings: &[S],
) -> Result<OccurrenceCounts<'_>, CountsError> {
    let required = preprocess(strings);
    let mut total_len = 0usize;
    let mut distinct = BTreeSet::new();
    for &word in &required {
        // Blocker phases use signed offsets, including negative positions.
        isize::try_from(word.len()).map_err(|_| CountsError::ArithmeticOverflow)?;
        total_len = add(total_len, word.len())?;
        for start in 0..word.len() {
            for end in start + 1..=word.len() {
                distinct.insert(&word[start..end]);
            }
        }
    }
    let mut words = Vec::new();
    words.try_reserve_exact(distinct.len())?;
    words.extend(distinct);

    let mut counts = filled(words.len(), 0usize)?;
    let mut left_extensions = filled(words.len(), 0usize)?;
    let mut right_extensions = filled(words.len(), 0usize)?;
    let mut rules = filled(words.len(), Vec::<PeriodRule<'_>>::new())?;
    let mut order = Vec::new();
    order.try_reserve_exact(words.len())?;
    order.extend(0..words.len());
    order.sort_unstable_by_key(|&id| (Reverse(words[id].len()), id));

    let mut lower_bound = 0;
    for id in order {
        let word = words[id];
        let mut count = usize::from(required.binary_search(&word).is_ok())
            .max(left_extensions[id])
            .max(right_extensions[id]);
        for rule in std::mem::take(&mut rules[id]) {
            let blocked = blocking(rule.template, word, &words, &counts)?;
            count = count.max(add(blocked, rule.steps + 1)?);
        }
        counts[id] = count;
        debug_assert!(count <= total_len, "lem:counts bounds every count by L");

        // A finalized word contributes to the extension sums of its immediate
        // suffix and prefix, which will both be processed later.
        if word.len() == 1 {
            lower_bound = add(lower_bound, count)?;
        } else {
            let suffix = word_id(&words, &word[1..]);
            let prefix = word_id(&words, &word[..word.len() - 1]);
            left_extensions[suffix] = add(left_extensions[suffix], count)?;
            right_extensions[prefix] = add(right_extensions[prefix], count)?;
        }

        // eq:period-rule includes nonleast periods, including periods that do
        // not divide the word's length. The first p bytes determine A on Z.
        for period in 1..word.len() {
            if !(period..word.len()).all(|i| word[i] == word[i % period]) {
                continue;
            }
            let template = &word[..period];
            if count <= blocking(template, word, &words, &counts)? {
                continue;
            }
            for steps in 1..=(word.len() - 1) / period {
                let prefix = word_id(&words, &word[..word.len() - steps * period]);
                rules[prefix].try_reserve(1)?;
                rules[prefix].push(PeriodRule { template, steps });
            }
            // The trigger is now final, but B_A(w; m) must be evaluated when w
            // is processed: its blockers can be shorter than this word.
        }
    }

    Ok(OccurrenceCounts {
        words,
        counts,
        lower_bound,
    })
}

#[derive(Clone, Copy)]
struct PeriodRule<'a> {
    template: &'a [u8],
    steps: usize,
}

fn filled<T: Clone>(len: usize, value: T) -> Result<Vec<T>, CountsError> {
    let mut result = Vec::new();
    result.try_reserve_exact(len)?;
    result.resize(len, value);
    Ok(result)
}

fn add(left: usize, right: usize) -> Result<usize, CountsError> {
    left.checked_add(right)
        .ok_or(CountsError::ArithmeticOverflow)
}

fn word_id(words: &[&[u8]], word: &[u8]) -> usize {
    words
        .binary_search(&word)
        .expect("nonempty substrings belong to the substring-closed word set")
}

// eq:block-functional: count placements, not just distinct bracketing words.
// Only strictly longer words contribute, so their counts are already final.
fn blocking(
    template: &[u8],
    word: &[u8],
    words: &[&[u8]],
    counts: &[usize],
) -> Result<usize, CountsError> {
    let mut total = 0;
    for (&bracket, &count) in words.iter().zip(counts) {
        if count == 0 || bracket.len().saturating_sub(word.len()) < 2 {
            continue;
        }
        let contribution = blocking_coefficient(template, word, bracket)
            .checked_mul(count)
            .ok_or(CountsError::ArithmeticOverflow)?;
        total = add(total, contribution)?;
    }
    Ok(total)
}

// eq:block-interior and eq:block-ends. Both endpoints must mismatch, and the
// entire interior must match A at the phase of the chosen occurrence.
fn blocking_coefficient(template: &[u8], word: &[u8], bracket: &[u8]) -> usize {
    if bracket.len().saturating_sub(word.len()) < 2 {
        return 0;
    }
    (1..bracket.len() - word.len())
        .filter(|&start| {
            let at = |index: usize| {
                let offset = index as isize - start as isize;
                template[offset.rem_euclid(template.len() as isize) as usize]
            };
            bracket[start..start + word.len()] == *word
                && bracket[0] != at(0)
                && bracket[bracket.len() - 1] != at(bracket.len() - 1)
                && (1..bracket.len() - 1).all(|index| bracket[index] == at(index))
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::blocking_coefficient;

    #[test]
    fn blockers_count_each_overlapping_placement() {
        assert_eq!(blocking_coefficient(b"a", b"aa", b"baaaab"), 3);
        assert_eq!(blocking_coefficient(b"ab", b"aba", b"aababaa"), 2);
        assert_eq!(blocking_coefficient(b"abc", b"abc", b"babcabcb"), 2);
    }

    #[test]
    fn blockers_require_matching_interior_and_two_mismatching_ends() {
        assert_eq!(blocking_coefficient(b"ab", b"aba", b"aababaa"), 2);
        assert_eq!(blocking_coefficient(b"ab", b"aba", b"bababaa"), 0);
        assert_eq!(blocking_coefficient(b"ab", b"aba", b"aababab"), 0);
        assert_eq!(blocking_coefficient(b"ab", b"aba", b"aababbaa"), 0);
        assert_eq!(blocking_coefficient(b"ab", b"aba", b"aba"), 0);
    }
}
