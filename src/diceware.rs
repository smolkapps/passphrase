//! Diceware passphrase generation.
//!
//! Words are chosen uniformly at random from the embedded EFF large wordlist.
//! The RNG is injected (`&mut impl Rng`) so production code can pass an OS
//! CSPRNG while tests pass a seeded `StdRng` for deterministic assertions.

use rand::seq::SliceRandom;
use rand::Rng;

/// Options controlling diceware passphrase generation.
#[derive(Debug, Clone)]
pub struct DicewareOptions {
    /// Number of words in the passphrase.
    pub words: usize,
    /// Separator placed between words (e.g. "-").
    pub separator: String,
    /// Capitalize the first letter of each word.
    pub capitalize: bool,
    /// Append a random decimal digit (0-9) to the end of the passphrase.
    pub add_number: bool,
    /// Append a random symbol from a fixed set to the end of the passphrase.
    pub add_symbol: bool,
}

impl Default for DicewareOptions {
    fn default() -> Self {
        Self {
            words: 6,
            separator: "-".to_string(),
            capitalize: false,
            add_number: false,
            add_symbol: false,
        }
    }
}

/// The fixed symbol set used by `--add-symbol`.
pub const ADD_SYMBOL_SET: &[char] = &[
    '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '+', '=',
];

/// Capitalize the first character of an ASCII word, leaving the rest unchanged.
fn capitalize_first(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
    }
}

/// Generate a diceware passphrase from `wordlist` using the injected `rng`.
///
/// Returns the assembled passphrase string. Each word is selected
/// independently and uniformly (`SliceRandom::choose`), so the generation
/// entropy is exactly `words * log2(wordlist.len())`.
pub fn generate<R: Rng + ?Sized>(opts: &DicewareOptions, wordlist: &[&str], rng: &mut R) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(opts.words);
    for _ in 0..opts.words {
        // `choose` returns None only on an empty slice; the wordlist is never
        // empty in practice, but fall back to "" to stay panic-free.
        let w = wordlist.choose(rng).copied().unwrap_or("");
        if opts.capitalize {
            parts.push(capitalize_first(w));
        } else {
            parts.push(w.to_string());
        }
    }

    let mut phrase = parts.join(&opts.separator);

    if opts.add_number {
        let digit = rng.gen_range(0..10u32);
        phrase.push_str(&digit.to_string());
    }
    if opts.add_symbol {
        let sym = ADD_SYMBOL_SET.choose(rng).copied().unwrap_or('!');
        phrase.push(sym);
    }

    phrase
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn list() -> Vec<&'static str> {
        crate::wordlist::words()
    }

    #[test]
    fn word_count_matches_request() {
        let wl = list();
        // Use `_` as the separator for counting: EFF words contain only [a-z]
        // and `-`, never `_`, so split('_') reliably counts words even when a
        // chosen word is itself hyphenated (e.g. `drop-down`).
        for n in [1usize, 3, 6, 10] {
            let opts = DicewareOptions {
                words: n,
                separator: "_".into(),
                ..Default::default()
            };
            let mut rng = StdRng::seed_from_u64(42);
            let phrase = generate(&opts, &wl, &mut rng);
            assert_eq!(
                phrase.split('_').count(),
                n,
                "phrase `{phrase}` should have {n} words"
            );
        }
    }

    #[test]
    fn separator_is_respected() {
        let wl = list();
        let opts = DicewareOptions {
            words: 4,
            separator: ".".into(),
            ..Default::default()
        };
        let mut rng = StdRng::seed_from_u64(7);
        let phrase = generate(&opts, &wl, &mut rng);
        // `.` never appears inside an EFF word, so dot-count == words - 1.
        assert_eq!(
            phrase.matches('.').count(),
            3,
            "4 words => 3 dots in `{phrase}`"
        );
    }

    #[test]
    fn empty_separator_concatenates() {
        let wl = list();
        let opts = DicewareOptions {
            words: 3,
            separator: "".into(),
            ..Default::default()
        };
        let mut rng = StdRng::seed_from_u64(1);
        let phrase = generate(&opts, &wl, &mut rng);
        // No separator: the words run together. (Words may themselves contain a
        // hyphen, so we only assert the result is a single non-empty token with
        // no separator artifact — not the absence of '-'.)
        assert!(!phrase.is_empty());
        assert!(!phrase.contains(' '));
    }

    #[test]
    fn seeded_rng_is_deterministic() {
        let wl = list();
        let opts = DicewareOptions {
            words: 6,
            separator: "-".into(),
            ..Default::default()
        };
        let mut a = StdRng::seed_from_u64(12345);
        let mut b = StdRng::seed_from_u64(12345);
        let pa = generate(&opts, &wl, &mut a);
        let pb = generate(&opts, &wl, &mut b);
        assert_eq!(pa, pb, "same seed must produce identical passphrase");
    }

    #[test]
    fn different_seeds_differ() {
        let wl = list();
        let opts = DicewareOptions {
            words: 6,
            separator: "-".into(),
            ..Default::default()
        };
        let mut a = StdRng::seed_from_u64(1);
        let mut b = StdRng::seed_from_u64(2);
        assert_ne!(generate(&opts, &wl, &mut a), generate(&opts, &wl, &mut b));
    }

    #[test]
    fn capitalize_uppercases_each_word_initial() {
        let wl = list();
        // `_` separator so split() yields whole words; capitalize only upper-
        // cases each word's FIRST letter, not letters after an internal hyphen,
        // so splitting on `-` would wrongly flag the back half of `drop-down`.
        let opts = DicewareOptions {
            words: 5,
            separator: "_".into(),
            capitalize: true,
            ..Default::default()
        };
        let mut rng = StdRng::seed_from_u64(99);
        let phrase = generate(&opts, &wl, &mut rng);
        for word in phrase.split('_') {
            let first = word.chars().next().unwrap();
            assert!(
                first.is_ascii_uppercase(),
                "word `{word}` should start uppercase"
            );
        }
    }

    #[test]
    fn add_number_appends_a_digit() {
        let wl = list();
        let opts = DicewareOptions {
            words: 3,
            separator: "-".into(),
            add_number: true,
            ..Default::default()
        };
        let mut rng = StdRng::seed_from_u64(5);
        let phrase = generate(&opts, &wl, &mut rng);
        let last = phrase.chars().last().unwrap();
        assert!(
            last.is_ascii_digit(),
            "expected trailing digit in `{phrase}`"
        );
    }

    #[test]
    fn add_symbol_appends_a_symbol() {
        let wl = list();
        let opts = DicewareOptions {
            words: 3,
            separator: "-".into(),
            add_symbol: true,
            ..Default::default()
        };
        let mut rng = StdRng::seed_from_u64(5);
        let phrase = generate(&opts, &wl, &mut rng);
        let last = phrase.chars().last().unwrap();
        assert!(
            ADD_SYMBOL_SET.contains(&last),
            "expected trailing symbol in `{phrase}`"
        );
    }

    #[test]
    fn capitalize_first_handles_empty() {
        assert_eq!(capitalize_first(""), "");
        assert_eq!(capitalize_first("a"), "A");
        assert_eq!(capitalize_first("abc"), "Abc");
    }
}
