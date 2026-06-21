//! Entropy estimation in bits.
//!
//! Entropy here is the *generation* entropy of the scheme, assuming every
//! selection is uniform and independent (which it is, because we draw from a
//! CSPRNG). It is the log2 of the size of the space the secret was drawn from.
//!
//! - Diceware: `words * log2(list_size)`. For the EFF large list
//!   (`list_size = 7776`), each word contributes `log2(7776) ≈ 12.925` bits.
//! - Random charset: `length * log2(pool_size)`.

/// Bits of entropy for a diceware passphrase.
///
/// `words` words each drawn uniformly from a list of `list_size` entries.
pub fn diceware_bits(words: usize, list_size: usize) -> f64 {
    if list_size <= 1 {
        return 0.0;
    }
    words as f64 * (list_size as f64).log2()
}

/// Bits of entropy for a random-character password.
///
/// `length` characters each drawn uniformly from a pool of `pool_size` symbols.
pub fn charset_bits(length: usize, pool_size: usize) -> f64 {
    if pool_size <= 1 {
        return 0.0;
    }
    length as f64 * (pool_size as f64).log2()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diceware_six_words_is_about_77_5_bits() {
        // log2(7776) = 12.9248125... ; * 6 = 77.5488752...
        let bits = diceware_bits(6, 7776);
        assert!(
            (bits - 77.5488).abs() < 0.01,
            "expected ~77.55 bits for 6 EFF words, got {bits}"
        );
    }

    #[test]
    fn diceware_per_word_is_about_12_925_bits() {
        let bits = diceware_bits(1, 7776);
        assert!((bits - 12.9248).abs() < 0.001, "got {bits}");
    }

    #[test]
    fn charset_full_ascii_printable_94_pool() {
        // log2(94) = 6.5546... ; * 20 = 131.09...
        let bits = charset_bits(20, 94);
        assert!((bits - 131.0937).abs() < 0.01, "got {bits}");
    }

    #[test]
    fn charset_lower_only_26_pool() {
        // log2(26) = 4.70044 ; * 8 = 37.6035
        let bits = charset_bits(8, 26);
        assert!((bits - 37.6035).abs() < 0.01, "got {bits}");
    }

    #[test]
    fn degenerate_pools_are_zero() {
        assert_eq!(diceware_bits(6, 1), 0.0);
        assert_eq!(diceware_bits(6, 0), 0.0);
        assert_eq!(charset_bits(20, 1), 0.0);
        assert_eq!(charset_bits(20, 0), 0.0);
    }

    #[test]
    fn zero_length_is_zero_bits() {
        assert_eq!(diceware_bits(0, 7776), 0.0);
        assert_eq!(charset_bits(0, 94), 0.0);
    }
}
