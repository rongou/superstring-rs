use superstring::{exact, greedy};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cases: &[(&str, &[&[u8]])] = &[
        ("empty", &[]),
        ("overlap chain", &[b"abc", b"bcd", b"cde"]),
        ("greedy tie", &[b"ab", b"ba", b"bb"]),
        ("periodic", &[b"abababa", b"bababab"]),
        (
            "five strings",
            &[b"catg", b"ctaagt", b"gcta", b"ttca", b"atgcatc"],
        ),
    ];
    println!(
        "{:<16} {:>7} {:>7} {:>14}",
        "Case", "Exact", "Greedy", "Greedy / exact"
    );
    for (name, inputs) in cases {
        let optimum = exact(inputs)?.len();
        let heuristic = greedy(inputs)?.len();
        let ratio = if optimum == 0 {
            "n/a".to_owned()
        } else {
            format!("{:.3}", heuristic as f64 / optimum as f64)
        };
        println!("{name:<16} {optimum:>7} {heuristic:>7} {ratio:>14}");
    }
    Ok(())
}
