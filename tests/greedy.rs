use superstring::{MAX_EXACT_STRINGS, exact, greedy};

mod common;
use common::{binary_words, covers, next_random};

fn check_against_exact(inputs: &[&[u8]]) {
    let output = greedy(inputs).unwrap();
    let optimum = exact(inputs).unwrap().len();
    assert!(
        covers(&output, inputs),
        "{output:?} does not cover {inputs:?}"
    );
    assert!(output.len() >= optimum, "input: {inputs:?}");
    assert!(output.len() <= inputs.iter().map(|word| word.len()).sum());

    let reversed: Vec<_> = inputs.iter().copied().rev().collect();
    assert_eq!(greedy(&reversed).unwrap(), output);
    let mut redundant = inputs.to_vec();
    redundant.extend_from_slice(inputs);
    redundant.push(b"");
    assert_eq!(greedy(&redundant).unwrap(), output);
}

#[test]
fn empty_and_redundant_inputs() {
    assert!(greedy::<&[u8]>(&[]).unwrap().is_empty());
    assert!(greedy(&[b"", b""]).unwrap().is_empty());
    assert_eq!(
        greedy(&[b"abc".as_slice(), b"bc", b"abc", b""]).unwrap(),
        b"abc"
    );
}

#[test]
fn directed_overlaps_and_repeated_merges() {
    // The winning pair can run opposite to the canonical order of its strings.
    assert_eq!(greedy(&[b"abc", b"zab"]).unwrap(), b"zabc");
    assert_eq!(greedy(&[b"cd", b"ab", b"bc"]).unwrap(), b"abcd");
    assert_eq!(greedy(&[b"ghi", b"abc", b"def"]).unwrap(), b"abcdefghi");
}

#[test]
fn tie_breaking_can_produce_a_suboptimal_superstring() {
    let inputs = [b"ab", b"ba", b"bb"];
    assert_eq!(greedy(&inputs).unwrap(), b"ababb");
    assert_eq!(exact(&inputs).unwrap().len(), 4);
    check_against_exact(&inputs.map(|word| word.as_slice()));
}

#[test]
fn arbitrary_bytes_and_owned_inputs() {
    let inputs = vec![vec![0xff, 0, 0x80], vec![0, 0x80, 0xfe], vec![0x80]];
    assert_eq!(greedy(&inputs).unwrap(), vec![0xff, 0, 0x80, 0xfe]);
}

#[test]
fn periodic_inputs_remain_covered() {
    let cases: &[&[&[u8]]] = &[
        &[b"aaaaa", b"aaa", b"a"],
        &[b"abababa", b"bababab", b"abacaba", b"bacabab"],
        &[b"aaaaab", b"baaaaa", b"aaaac", b"caaaa"],
        &[b"abcabcab", b"bcabcabc", b"cabcabca", b"abcabd", b"dabcab"],
    ];
    for inputs in cases {
        check_against_exact(inputs);
    }
}

#[test]
fn accepts_more_strings_than_the_exact_solver() {
    let inputs: Vec<_> = (0..=MAX_EXACT_STRINGS)
        .map(|value| vec![value as u8])
        .collect();
    let expected: Vec<u8> = inputs.iter().flatten().copied().collect();
    assert_eq!(greedy(&inputs).unwrap(), expected);
}

#[test]
fn exhaustive_tiny_instances_are_valid_and_deterministic() {
    // The same 576 tiny instances used to independently validate the exact solver.
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
fn seeded_instances_are_valid_and_deterministic() {
    let mut seed = 0x05c5_6eed;
    let alphabet = [0, 1, 0x80, 0xff];
    for _ in 0..128 {
        let inputs: Vec<Vec<u8>> = (0..next_random(&mut seed) % 9)
            .map(|_| {
                (0..next_random(&mut seed) % 8)
                    .map(|_| alphabet[next_random(&mut seed) % alphabet.len()])
                    .collect()
            })
            .collect();
        let borrowed: Vec<_> = inputs.iter().map(Vec::as_slice).collect();
        check_against_exact(&borrowed);
    }
}
