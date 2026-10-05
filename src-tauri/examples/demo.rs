use openlex_lib::{lexique::Lexique, predicteur::suggerer};
use std::time::Instant;

fn main() {
    let lex = Lexique::global();
    for s in std::env::args().skip(1) {
        let t = Instant::now();
        let r = suggerer(lex, &s, 6);
        let mots: Vec<_> = r
            .iter()
            .map(|x| format!("{} ({})", x.mot, x.score))
            .collect();
        println!("{s:>12} [{:?}] → {}", t.elapsed(), mots.join(", "));
    }
}
