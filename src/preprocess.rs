/// Removes empty strings, duplicates, and strings contained in another input.
///
/// Returns borrowed slices in ascending bytewise lexicographic order. The
/// remaining strings have exactly the same common superstrings as the original
/// inputs, including when every input is empty. No string contents are copied.
///
/// This is the preprocessing in Section 2 of the family 128 paper (the start of
/// `build/sections/02-counts.tex`). The straightforward containment scan favors
/// readability over performance on large collections.
///
/// ```
/// use superstring::preprocess;
///
/// let inputs = [b"bc".as_slice(), b"abc", b"", b"abc", b"xyz"];
/// assert_eq!(preprocess(&inputs), vec![b"abc".as_slice(), b"xyz"]);
/// ```
pub fn preprocess<S: AsRef<[u8]>>(strings: &[S]) -> Vec<&[u8]> {
    let mut words: Vec<&[u8]> = strings
        .iter()
        .map(|word| word.as_ref())
        .filter(|word| !word.is_empty())
        .collect();
    words.sort_unstable();
    words.dedup();

    words
        .iter()
        .copied()
        .filter(|word| {
            !words.iter().any(|other| {
                other.len() > word.len() && other.windows(word.len()).any(|window| window == *word)
            })
        })
        .collect()
}
