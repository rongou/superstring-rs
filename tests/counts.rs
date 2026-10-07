use std::collections::BTreeSet;

use superstring::{OccurrenceCounts, exact, greedy, occurrence_counts};

mod common;
use common::{binary_words, covers, next_random};

fn occurrences(text: &[u8], word: &[u8]) -> usize {
    text.windows(word.len())
        .filter(|&window| window == word)
        .count()
}

fn check_occurrences(counts: &OccurrenceCounts<'_>, text: &[u8]) {
    for (word, count) in counts.entries() {
        assert!(
            count <= occurrences(text, word),
            "m({word:?}) = {count} exceeds its occurrences in {text:?}"
        );
    }
}

// An independent, literal version of eq:block-functional for checking the final
// recurrence. Materialize each periodic window using unsigned cyclic iteration,
// rather than the implementation's signed indexing and deferred rules.
fn reference_blocking(counts: &OccurrenceCounts<'_>, pattern: &[u8], word: &[u8]) -> usize {
    let mut total = 0;
    for (bracket, count) in counts.entries() {
        if bracket.len() < word.len() + 2 {
            continue;
        }
        for start in 1..bracket.len() - word.len() {
            if bracket[start..start + word.len()] != *word {
                continue;
            }
            let phase = (pattern.len() - start % pattern.len()) % pattern.len();
            let window: Vec<_> = pattern
                .iter()
                .copied()
                .cycle()
                .skip(phase)
                .take(bracket.len())
                .collect();
            let end = bracket.len() - 1;
            if bracket[0] != window[0]
                && bracket[end] != window[end]
                && bracket[1..end] == window[1..end]
            {
                total += count;
            }
        }
    }
    total
}

fn check_recurrence(inputs: &[&[u8]], counts: &OccurrenceCounts<'_>) {
    for (word, count) in counts.entries() {
        let left: usize = counts
            .entries()
            .filter(|(other, _)| other.len() == word.len() + 1 && other.ends_with(word))
            .map(|(_, value)| value)
            .sum();
        let right: usize = counts
            .entries()
            .filter(|(other, _)| other.len() == word.len() + 1 && other.starts_with(word))
            .map(|(_, value)| value)
            .sum();
        // Contained inputs also have a positive count by extension, so requiring
        // them here is equivalent to using only the preprocessed inputs.
        let mut expected = usize::from(inputs.contains(&word)).max(left).max(right);
        for (longer, longer_count) in counts.entries() {
            if longer.len() <= word.len() || !longer.starts_with(word) {
                continue;
            }
            let gap = longer.len() - word.len();
            for period in 1..longer.len() {
                let pattern = &longer[..period];
                if gap % period != 0 || !longer.iter().eq(pattern.iter().cycle().take(longer.len()))
                {
                    continue;
                }
                if longer_count > reference_blocking(counts, pattern, longer) {
                    expected =
                        expected.max(reference_blocking(counts, pattern, word) + gap / period + 1);
                }
            }
        }
        assert_eq!(count, expected, "input: {inputs:?}, word: {word:?}");
    }
}

fn check_against_exact(inputs: &[&[u8]]) {
    let counts = occurrence_counts(inputs).unwrap();
    let optimum = exact(inputs).unwrap();
    assert!(covers(&optimum, inputs));
    assert!(counts.lower_bound() <= optimum.len(), "input: {inputs:?}");
    assert_eq!(
        counts.lower_bound(),
        counts
            .entries()
            .filter(|(word, _)| word.len() == 1)
            .map(|(_, count)| count)
            .sum()
    );

    let mut substrings = BTreeSet::new();
    for word in inputs {
        for start in 0..word.len() {
            for end in start + 1..=word.len() {
                substrings.insert(&word[start..end]);
            }
        }
    }
    assert!(counts.entries().map(|(word, _)| word).eq(substrings));
    for (word, count) in counts.entries() {
        assert!(count > 0);
        assert_eq!(counts.count(word), count);
    }
    assert_eq!(counts.count(b""), 0);
    check_occurrences(&counts, &optimum);
    check_occurrences(&counts, &greedy(inputs).unwrap());
    check_occurrences(&counts, &inputs.concat());
    check_recurrence(inputs, &counts);

    let reversed: Vec<_> = inputs.iter().copied().rev().collect();
    assert_eq!(occurrence_counts(&reversed).unwrap(), counts);
    let mut redundant = inputs.to_vec();
    redundant.extend_from_slice(inputs);
    redundant.push(b"");
    assert_eq!(occurrence_counts(&redundant).unwrap(), counts);
}

#[test]
fn empty_and_redundant_inputs() {
    let empty: [&[u8]; 0] = [];
    let counts = occurrence_counts(&empty).unwrap();
    assert_eq!(counts.lower_bound(), 0);
    assert_eq!(counts.entries().len(), 0);
    assert_eq!(counts.count(b"absent"), 0);
    check_against_exact(&[]);
    check_against_exact(&[b"", b""]);
    check_against_exact(&[b"abc", b"bc", b"abc", b"", b"xyz"]);
    assert_eq!(
        occurrence_counts(&[b"abc".as_slice(), b"bc", b"abc", b""]).unwrap(),
        occurrence_counts(&[b"abc"]).unwrap()
    );
}

#[test]
fn unary_powers_force_every_overlapping_occurrence() {
    // Section 2's a^h example: extension inequalities alone give only one.
    for len in 1..=10 {
        let word = vec![b'a'; len];
        let inputs = [&word];
        let counts = occurrence_counts(&inputs).unwrap();
        assert_eq!(counts.lower_bound(), len);
        for prefix_len in 1..=len {
            assert_eq!(counts.count(&word[..prefix_len]), len - prefix_len + 1);
        }
        check_against_exact(&[&word]);
    }
}

#[test]
fn fully_blocked_occurrences_do_not_trigger_period_rules() {
    let inputs = [b"baaaaab"];
    let counts = occurrence_counts(&inputs).unwrap();
    // B_A(aaaaa; m) = m(baaaaab) = m(aaaaa) = 1. The trigger is strictly >.
    assert_eq!(counts.lower_bound(), 7);
    assert_eq!(counts.count(b"a"), 5);
    assert_eq!(counts.count(b"aa"), 4);
    assert_eq!(counts.count(b"aaa"), 3);
    assert_eq!(counts.count(b"aaaa"), 2);
    check_against_exact(&[b"baaaaab"]);
}

#[test]
fn shorter_blockers_are_finalized_before_a_rule_is_applied() {
    let inputs: &[&[u8]] = &[b"aaaaaaaa", b"baaaab"];
    let counts = occurrence_counts(inputs).unwrap();
    // The long unblocked run forces eight a's, plus four occurrences blocked by
    // baaaab. That blocker is shorter than the word which triggers the rule.
    assert_eq!(counts.lower_bound(), 14);
    assert_eq!(counts.count(b"a"), 12);
    assert_eq!(counts.count(b"aa"), 10);
    assert_eq!(counts.count(b"aaa"), 8);
    assert_eq!(counts.count(b"aaaa"), 6);
    check_against_exact(inputs);
}

#[test]
fn periodic_phases_and_competing_words() {
    let cases: &[&[&[u8]]] = &[
        &[b"ababababa", b"aababaa"],
        &[b"abababa", b"bababab", b"abacaba", b"bacabab"],
        &[b"aaaaab", b"baaaaa", b"aaaac", b"caaaa"],
        &[b"abcabcab", b"bcabcabc", b"cabcabca", b"babcabcb"],
    ];
    for inputs in cases {
        check_against_exact(inputs);
    }
    let counts = occurrence_counts(cases[0]).unwrap();
    assert_eq!(counts.lower_bound(), 14);
    assert_eq!(counts.count(b"a"), 8);
    assert_eq!(counts.count(b"b"), 6);
    assert_eq!(counts.count(b"aba"), 6);
    assert_eq!(counts.count(b"bab"), 4);
}

#[test]
fn lower_bound_can_be_strictly_less_than_optimum() {
    let inputs = [b"ab", b"ba", b"bb"];
    let counts = occurrence_counts(&inputs).unwrap();
    assert_eq!(counts.lower_bound(), 3);
    assert_eq!(counts.count(b"a"), 1);
    assert_eq!(counts.count(b"b"), 2);
    assert_eq!(exact(&inputs).unwrap().len(), 4);
    assert_eq!(counts.count(b"missing"), 0);
}

#[test]
fn byte_relabeling_preserves_counts_and_lower_bound() {
    let inputs: &[&[u8]] = &[b"abcabcab", b"babcabcb", b"aaaaa"];
    let rename = |word: &[u8]| -> Vec<u8> {
        word.iter()
            .map(|&byte| [0xff, 0, 0x80][usize::from(byte - b'a')])
            .collect()
    };
    let renamed: Vec<_> = inputs.iter().map(|word| rename(word)).collect();
    let original_counts = occurrence_counts(inputs).unwrap();
    let renamed_counts = occurrence_counts(&renamed).unwrap();
    assert_eq!(original_counts.lower_bound(), renamed_counts.lower_bound());
    for (word, count) in original_counts.entries() {
        assert_eq!(renamed_counts.count(&rename(word)), count);
    }
    check_against_exact(&renamed.iter().map(Vec::as_slice).collect::<Vec<_>>());
}

#[test]
fn exhaustive_tiny_instances_satisfy_the_recurrence_and_bound() {
    // The same 576 instances used to independently validate the exact solver.
    let words = binary_words(3);
    check_against_exact(&[]);
    for (i, a) in words.iter().enumerate() {
        check_against_exact(&[a]);
        for (j, b) in words.iter().enumerate().skip(i + 1) {
            check_against_exact(&[a, b]);
            for c in words.iter().skip(j + 1) {
                check_against_exact(&[a, b, c]);
            }
        }
    }
}

#[test]
fn counts_bound_occurrences_in_every_tiny_superstring() {
    let inputs = binary_words(3);
    let candidates = binary_words(6);
    for (i, a) in inputs.iter().enumerate() {
        for b in inputs.iter().skip(i) {
            let instance = [a.as_slice(), b.as_slice()];
            let counts = occurrence_counts(&instance).unwrap();
            for text in candidates.iter().filter(|text| covers(text, &instance)) {
                check_occurrences(&counts, text);
                assert!(counts.lower_bound() <= text.len());
            }
        }
    }
}

#[test]
fn seeded_instances_satisfy_the_recurrence_and_bound() {
    let mut seed = 0xc017_6eed;
    let alphabet = [0, 0x80, 0xff];
    for _ in 0..128 {
        let inputs: Vec<Vec<u8>> = (0..next_random(&mut seed) % 7)
            .map(|_| {
                (0..next_random(&mut seed) % 8)
                    .map(|_| alphabet[next_random(&mut seed) % alphabet.len()])
                    .collect()
            })
            .collect();
        check_against_exact(&inputs.iter().map(Vec::as_slice).collect::<Vec<_>>());
    }
}
