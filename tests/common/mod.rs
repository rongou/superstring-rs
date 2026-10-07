pub fn contains(text: &[u8], word: &[u8]) -> bool {
    word.is_empty() || text.windows(word.len()).any(|window| window == word)
}

pub fn covers(text: &[u8], inputs: &[&[u8]]) -> bool {
    inputs.iter().all(|word| contains(text, word))
}

pub fn binary_words(max_len: usize) -> Vec<Vec<u8>> {
    (0..=max_len)
        .flat_map(|len| {
            (0..1usize << len).map(move |bits| {
                (0..len)
                    .map(|offset| b'a' + ((bits >> (len - 1 - offset)) & 1) as u8)
                    .collect()
            })
        })
        .collect()
}

pub fn next_random(state: &mut u64) -> usize {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    (*state >> 32) as usize
}
