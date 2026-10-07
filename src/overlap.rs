/// Length of the longest suffix of `left` that is a prefix of `right`.
pub(crate) fn overlap(left: &[u8], right: &[u8]) -> usize {
    (1..=left.len().min(right.len()))
        .rev()
        .find(|&len| left[left.len() - len..] == right[..len])
        .unwrap_or(0)
}
