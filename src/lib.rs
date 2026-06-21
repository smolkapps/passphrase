//! `passphrase` — a secure, offline passphrase and password generator.
//!
//! The crate is split into small, independently testable modules:
//!
//! - [`wordlist`]: the embedded EFF large wordlist (7776 words).
//! - [`diceware`]: diceware passphrase generation over an injected RNG.
//! - [`charset`]: random-character password generation with class controls.
//! - [`entropy`]: entropy-in-bits estimation for both schemes.
//!
//! Every generator takes the RNG as `&mut impl Rng`, so production code passes
//! an OS CSPRNG (`rand::rngs::OsRng`) while tests pass a seeded `StdRng` for
//! deterministic assertions. The library performs **no** I/O and **no** network
//! access; the wordlist is embedded at compile time with `include_str!`.

pub mod charset;
pub mod diceware;
pub mod entropy;
pub mod wordlist;

pub use charset::{CharsetError, CharsetOptions};
pub use diceware::DicewareOptions;
