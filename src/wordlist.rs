//! The bundled EFF large wordlist (7776 words), embedded at compile time.
//!
//! The list is the EFF "large" list intended for 5-dice diceware
//! (6^5 = 7776 entries). It is embedded with `include_str!` so the binary is
//! fully self-contained and never touches the filesystem or network at runtime.

/// The raw embedded wordlist text: one lowercase ASCII word per line.
pub const RAW: &str = include_str!("eff_large_wordlist.txt");

/// Number of words a correct EFF large wordlist must contain (6^5).
pub const EXPECTED_LEN: usize = 7776;

/// Parse the embedded wordlist into a borrowed slice-backed `Vec<&str>`.
///
/// Trailing/leading whitespace per line is trimmed and blank lines are skipped,
/// so a stray trailing newline in the data file is harmless.
pub fn words() -> Vec<&'static str> {
    RAW.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn has_exactly_7776_words() {
        assert_eq!(
            words().len(),
            EXPECTED_LEN,
            "EFF large wordlist must be 7776 words"
        );
    }

    #[test]
    fn all_words_unique() {
        let w = words();
        let set: HashSet<&str> = w.iter().copied().collect();
        assert_eq!(set.len(), w.len(), "wordlist contains duplicate entries");
    }

    #[test]
    fn all_words_lowercase_ascii_no_spaces() {
        // The EFF large wordlist is all lowercase ASCII letters, with the sole
        // exception of four legitimately hyphenated entries (`drop-down`,
        // `felt-tip`, `t-shirt`, `yo-yo`). Allow `[a-z-]`; forbid uppercase,
        // digits, whitespace, and any non-ASCII byte.
        for word in words() {
            assert!(!word.is_empty(), "empty word found");
            assert!(word.is_ascii(), "word `{word}` contains a non-ASCII byte");
            assert!(
                word.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "word `{word}` has a char outside [a-z-]"
            );
            assert!(
                !word.chars().any(|c| c.is_whitespace()),
                "word `{word}` contains whitespace"
            );
        }
    }

    #[test]
    fn exactly_four_hyphenated_entries() {
        // Pin the known EFF hyphenated words so an accidental data swap is caught.
        let hyphenated: Vec<&str> = words().into_iter().filter(|w| w.contains('-')).collect();
        assert_eq!(
            hyphenated,
            vec!["drop-down", "felt-tip", "t-shirt", "yo-yo"],
            "unexpected set of hyphenated words"
        );
    }
}
