//! Random-character password generation with character-class controls.
//!
//! The caller enables any combination of classes (upper / lower / digits /
//! symbols). The enabled classes are unioned into a single pool and each
//! character is drawn uniformly from that pool using the injected RNG, so the
//! generation entropy is `length * log2(pool_size)`.
//!
//! `no_ambiguous` removes the visually confusable characters `0 O o l 1 I |`
//! from the pool before generation.

use rand::seq::SliceRandom;
use rand::Rng;

/// Uppercase ASCII letters.
pub const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
/// Lowercase ASCII letters.
pub const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
/// Decimal digits.
pub const DIGITS: &str = "0123456789";
/// Symbols (the printable-ASCII punctuation set).
pub const SYMBOLS: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

/// The exact ambiguous set excluded by `--no-ambiguous`: `0 O o l 1 I |`.
pub const AMBIGUOUS: &[char] = &['0', 'O', 'o', 'l', '1', 'I', '|'];

/// Which character classes to include in the pool.
#[derive(Debug, Clone, Copy)]
pub struct CharsetOptions {
    pub length: usize,
    pub upper: bool,
    pub lower: bool,
    pub digits: bool,
    pub symbols: bool,
    pub no_ambiguous: bool,
}

impl Default for CharsetOptions {
    fn default() -> Self {
        // A sensible default password: upper+lower+digits, length 16.
        Self {
            length: 16,
            upper: true,
            lower: true,
            digits: true,
            symbols: false,
            no_ambiguous: false,
        }
    }
}

/// Errors that can arise when building a character pool.
#[derive(Debug, PartialEq, Eq)]
pub enum CharsetError {
    /// No character class was enabled, so the pool would be empty.
    NoClassEnabled,
    /// Every selected character was ambiguous and got filtered out.
    EmptyAfterFilter,
    /// A zero-length password was requested.
    ZeroLength,
}

impl std::fmt::Display for CharsetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharsetError::NoClassEnabled => write!(
                f,
                "no character class enabled: pass at least one of --upper/--lower/--digits/--symbols"
            ),
            CharsetError::EmptyAfterFilter => {
                write!(f, "character pool is empty after removing ambiguous characters")
            }
            CharsetError::ZeroLength => write!(f, "password length must be at least 1"),
        }
    }
}

impl std::error::Error for CharsetError {}

/// Build the deduplicated character pool implied by `opts`.
///
/// Returns the pool as a `Vec<char>` (stable order: upper, lower, digits,
/// symbols) with ambiguous characters removed when requested. The pool's length
/// is what feeds the entropy calculation, so it is deduplicated to be exact.
pub fn build_pool(opts: &CharsetOptions) -> Result<Vec<char>, CharsetError> {
    let mut pool: Vec<char> = Vec::new();
    if opts.upper {
        pool.extend(UPPER.chars());
    }
    if opts.lower {
        pool.extend(LOWER.chars());
    }
    if opts.digits {
        pool.extend(DIGITS.chars());
    }
    if opts.symbols {
        pool.extend(SYMBOLS.chars());
    }
    if pool.is_empty() {
        return Err(CharsetError::NoClassEnabled);
    }

    if opts.no_ambiguous {
        pool.retain(|c| !AMBIGUOUS.contains(c));
    }

    // Dedup while preserving order (symbol set has no overlap with letters/digits,
    // but this keeps pool_size exact regardless of class definitions).
    let mut seen = std::collections::HashSet::new();
    pool.retain(|c| seen.insert(*c));

    if pool.is_empty() {
        return Err(CharsetError::EmptyAfterFilter);
    }
    Ok(pool)
}

/// Generate a random-character password from `opts` using the injected `rng`.
///
/// Each character is drawn uniformly and independently from the pool. Returns
/// the password plus the pool size used (so the caller can compute entropy
/// without rebuilding the pool).
pub fn generate<R: Rng + ?Sized>(
    opts: &CharsetOptions,
    rng: &mut R,
) -> Result<(String, usize), CharsetError> {
    if opts.length == 0 {
        return Err(CharsetError::ZeroLength);
    }
    let pool = build_pool(opts)?;
    let mut out = String::with_capacity(opts.length);
    for _ in 0..opts.length {
        // Safe: pool is non-empty here.
        out.push(*pool.choose(rng).unwrap());
    }
    Ok((out, pool.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn length_matches_request() {
        for len in [1usize, 8, 20, 64] {
            let opts = CharsetOptions {
                length: len,
                ..Default::default()
            };
            let mut rng = StdRng::seed_from_u64(3);
            let (pw, _) = generate(&opts, &mut rng).unwrap();
            assert_eq!(
                pw.chars().count(),
                len,
                "password `{pw}` should be {len} chars"
            );
        }
    }

    #[test]
    fn only_lower_produces_only_lowercase() {
        let opts = CharsetOptions {
            length: 200,
            upper: false,
            lower: true,
            digits: false,
            symbols: false,
            no_ambiguous: false,
        };
        let mut rng = StdRng::seed_from_u64(8);
        let (pw, pool) = generate(&opts, &mut rng).unwrap();
        assert_eq!(pool, 26);
        assert!(
            pw.chars().all(|c| c.is_ascii_lowercase()),
            "`{pw}` not all lowercase"
        );
    }

    #[test]
    fn only_digits_produces_only_digits() {
        let opts = CharsetOptions {
            length: 200,
            upper: false,
            lower: false,
            digits: true,
            symbols: false,
            no_ambiguous: false,
        };
        let mut rng = StdRng::seed_from_u64(8);
        let (pw, pool) = generate(&opts, &mut rng).unwrap();
        assert_eq!(pool, 10);
        assert!(
            pw.chars().all(|c| c.is_ascii_digit()),
            "`{pw}` not all digits"
        );
    }

    #[test]
    fn full_pool_size_is_94() {
        let opts = CharsetOptions {
            length: 1,
            upper: true,
            lower: true,
            digits: true,
            symbols: true,
            no_ambiguous: false,
        };
        let pool = build_pool(&opts).unwrap();
        // 26 + 26 + 10 + 32 = 94
        assert_eq!(pool.len(), 94);
    }

    #[test]
    fn no_class_enabled_errors() {
        let opts = CharsetOptions {
            length: 10,
            upper: false,
            lower: false,
            digits: false,
            symbols: false,
            no_ambiguous: false,
        };
        assert_eq!(build_pool(&opts), Err(CharsetError::NoClassEnabled));
        assert_eq!(
            generate(&opts, &mut StdRng::seed_from_u64(1)),
            Err(CharsetError::NoClassEnabled)
        );
    }

    #[test]
    fn zero_length_errors() {
        let opts = CharsetOptions {
            length: 0,
            ..Default::default()
        };
        assert_eq!(
            generate(&opts, &mut StdRng::seed_from_u64(1)),
            Err(CharsetError::ZeroLength)
        );
    }

    #[test]
    fn no_ambiguous_excludes_the_exact_set() {
        // Enable all classes so every ambiguous char would otherwise be present.
        let opts = CharsetOptions {
            length: 5000,
            upper: true,
            lower: true,
            digits: true,
            symbols: true,
            no_ambiguous: true,
        };
        // Pool must contain none of the ambiguous chars.
        let pool = build_pool(&opts).unwrap();
        for amb in AMBIGUOUS {
            assert!(!pool.contains(amb), "pool still contains ambiguous `{amb}`");
        }
        // Generated output (large sample) must contain none of them either.
        let mut rng = StdRng::seed_from_u64(2024);
        let (pw, used_pool) = generate(&opts, &mut rng).unwrap();
        for amb in AMBIGUOUS {
            assert!(!pw.contains(*amb), "password contains ambiguous `{amb}`");
        }
        // 94 total minus the 7 ambiguous chars that are actually in the pool
        // (0 O o l 1 I are letters/digits, | is a symbol => all 7 present) = 87.
        assert_eq!(
            used_pool, 87,
            "expected pool of 87 after removing 7 ambiguous chars"
        );
    }

    #[test]
    fn no_ambiguous_keeps_pipe_out_when_symbols_on() {
        let opts = CharsetOptions {
            length: 1,
            upper: false,
            lower: false,
            digits: false,
            symbols: true,
            no_ambiguous: true,
        };
        let pool = build_pool(&opts).unwrap();
        assert!(!pool.contains(&'|'), "pipe should be excluded as ambiguous");
        // SYMBOLS has 32 chars, only `|` is ambiguous => 31 remain.
        assert_eq!(pool.len(), 31);
    }

    #[test]
    fn seeded_rng_is_deterministic() {
        let opts = CharsetOptions {
            length: 20,
            ..Default::default()
        };
        let mut a = StdRng::seed_from_u64(555);
        let mut b = StdRng::seed_from_u64(555);
        assert_eq!(
            generate(&opts, &mut a).unwrap(),
            generate(&opts, &mut b).unwrap()
        );
    }

    #[test]
    fn membership_respects_enabled_classes_combo() {
        // upper + digits only.
        let opts = CharsetOptions {
            length: 300,
            upper: true,
            lower: false,
            digits: true,
            symbols: false,
            no_ambiguous: false,
        };
        let mut rng = StdRng::seed_from_u64(77);
        let (pw, pool) = generate(&opts, &mut rng).unwrap();
        assert_eq!(pool, 36);
        assert!(
            pw.chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()),
            "`{pw}` contains a char outside upper+digits"
        );
        assert!(!pw.chars().any(|c| c.is_ascii_lowercase()));
    }
}
