use superstring::{ExactError, MAX_EXACT_STRINGS, exact, preprocess};

mod common;
use common::{binary_words, contains, covers, next_random};

fn assert_exact(inputs: &[&[u8]], optimum: usize) {
    let output = exact(inputs).unwrap();
    assert!(
        covers(&output, inputs),
        "{output:?} does not cover {inputs:?}"
    );
    assert_eq!(
        output.len(),
        optimum,
        "input: {inputs:?}; output: {output:?}"
    );
    let reversed: Vec<_> = inputs.iter().copied().rev().collect();
    assert_eq!(exact(&reversed).unwrap(), output);
}

#[test]
fn preprocessing_removes_redundancy_and_orders_bytes() {
    let inputs: &[&[u8]] = &[
        b"bc", b"abc", b"", b"abc", b"a", b"c", b"z", b"\0\xff", b"\xff",
    ];
    let expected: Vec<&[u8]> = vec![b"\0\xff", b"abc", b"z"];
    assert_eq!(preprocess(inputs), expected);
    let reversed: Vec<_> = inputs.iter().copied().rev().collect();
    assert_eq!(preprocess(&reversed), expected);
}

#[test]
fn empty_and_redundant_inputs() {
    assert_exact(&[], 0);
    assert_exact(&[b"", b""], 0);
    assert_exact(&[b"abc", b"", b"bc", b"abc"], 3);
}

#[test]
fn overlaps_and_disjoint_strings() {
    assert_exact(&[b"abc", b"bcd"], 4);
    assert_exact(&[b"abc", b"def", b"ghi"], 9);
    assert_exact(&[b"ab", b"bc", b"ca"], 4);
    assert_exact(&[b"catg", b"ctaagt", b"gcta", b"ttca", b"atgcatc"], 16);
}

#[test]
fn arbitrary_bytes_and_owned_inputs() {
    let inputs = vec![vec![0xff, 0, 0x80], vec![0, 0x80, 0xfe], vec![0x80]];
    let output = exact(&inputs).unwrap();
    assert_eq!(output, vec![0xff, 0, 0x80, 0xfe]);
}

#[test]
fn repeated_and_competing_periods() {
    assert_exact(&[b"aaaaa", b"aaa", b"a"], 5);
    assert_exact(&[b"abababa", b"bababab"], 8);
    let cases: &[&[&[u8]]] = &[
        &[b"abababa", b"bababab", b"abacaba", b"bacabab"],
        &[b"aaaaab", b"baaaaa", b"aaaac", b"caaaa"],
        &[b"abcabcab", b"bcabcabc", b"cabcabca", b"abcabd", b"dabcab"],
    ];
    for inputs in cases {
        assert_exact(inputs, permutation_optimum(inputs));
    }
}

#[test]
fn resource_limit_is_checked_after_preprocessing() {
    let inputs: Vec<Vec<u8>> = (0..=MAX_EXACT_STRINGS)
        .map(|value| vec![value as u8])
        .collect();
    match exact(&inputs).unwrap_err() {
        ExactError::TooManyStrings { count, limit } => {
            assert_eq!(count, MAX_EXACT_STRINGS + 1);
            assert_eq!(limit, MAX_EXACT_STRINGS);
        }
        error => panic!("unexpected error: {error}"),
    }

    let redundant = vec![b"abc"; MAX_EXACT_STRINGS * 2];
    assert_eq!(exact(&redundant).unwrap(), b"abc");
}

#[test]
fn exhaustive_tiny_instances_match_literal_superstring_search() {
    // All 576 sets of up to three binary words of length at most three.
    // The oracle enumerates actual output strings in increasing length; it uses
    // neither preprocessing nor overlap paths nor subset dynamic programming.
    let words = binary_words(3);
    let candidates = binary_words(9);
    let check = |inputs: &[&[u8]]| {
        let optimum = candidates
            .iter()
            .find(|candidate| covers(candidate, inputs))
            .expect("concatenation has length at most nine")
            .len();
        assert_exact(inputs, optimum);
    };
    check(&[]);
    for (i, a) in words.iter().enumerate() {
        check(&[a]);
        for (j, b) in words.iter().enumerate().skip(i + 1) {
            check(&[a, b]);
            for c in words.iter().skip(j + 1) {
                check(&[a, b, c]);
            }
        }
    }
}

fn permutation_optimum(inputs: &[&[u8]]) -> usize {
    fn visit(inputs: &[&[u8]], used: &mut [bool], text: &[u8], remaining: usize, best: &mut usize) {
        if text.len() >= *best {
            return;
        }
        if remaining == 0 {
            *best = text.len();
            return;
        }
        for (i, word) in inputs.iter().enumerate() {
            if used[i] {
                continue;
            }
            used[i] = true;
            let mut merged = text.to_vec();
            if !contains(text, word) {
                let mut shared = 0;
                for len in 0..=text.len().min(word.len()) {
                    if text[text.len() - len..] == word[..len] {
                        shared = len;
                    }
                }
                merged.extend_from_slice(&word[shared..]);
            }
            visit(inputs, used, &merged, remaining - 1, best);
            used[i] = false;
        }
    }

    // Search every input ordering, skipping strings already contained in the
    // accumulated text. No library preprocessing or DP is used in this oracle.
    let mut best = inputs.iter().map(|word| word.len()).sum();
    visit(
        inputs,
        &mut vec![false; inputs.len()],
        &[],
        inputs.len(),
        &mut best,
    );
    best
}

#[test]
fn seeded_instances_match_permutation_search() {
    let mut seed = 0x5c5_128_u64;
    let alphabet = [0, 1, 0x80, 0xff];
    for _ in 0..128 {
        let inputs: Vec<Vec<u8>> = (0..next_random(&mut seed) % 8)
            .map(|_| {
                (0..next_random(&mut seed) % 7)
                    .map(|_| alphabet[next_random(&mut seed) % alphabet.len()])
                    .collect()
            })
            .collect();
        let borrowed: Vec<_> = inputs.iter().map(Vec::as_slice).collect();
        assert_exact(&borrowed, permutation_optimum(&borrowed));
    }
}
