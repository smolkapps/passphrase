//! `passphrase` CLI — thin wrapper over the library.
//!
//! Default (no subcommand): generate diceware passphrase(s).
//!   passphrase --words 6 --sep -
//! Subcommand `pw`: generate random-character password(s).
//!   passphrase pw --length 20 --upper --lower --digits --symbols --no-ambiguous
//!
//! Always prints an entropy estimate in bits unless `--quiet` is given.
//! Production randomness comes from the OS CSPRNG (`OsRng`).

use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use rand::rngs::OsRng;

use passphrase::charset::{self, CharsetOptions};
use passphrase::diceware::{self, DicewareOptions};
use passphrase::{entropy, wordlist};

#[derive(Parser, Debug)]
#[command(
    name = "passphrase",
    version,
    about = "Secure offline passphrase (diceware) and password generator",
    long_about = "Generates strong, memorable diceware passphrases from the bundled \
EFF large wordlist (7776 words), or random-character passwords with character-class \
controls. All randomness comes from the operating system CSPRNG; no network access."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Diceware options (used when no subcommand is given).
    #[command(flatten)]
    diceware: DicewareArgs,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Generate a random-character password (not word-based).
    Pw(PwArgs),
}

#[derive(Args, Debug)]
struct DicewareArgs {
    /// Number of words in the passphrase.
    #[arg(short, long, default_value_t = 6)]
    words: usize,

    /// Separator placed between words.
    #[arg(long = "sep", default_value = "-")]
    sep: String,

    /// Capitalize the first letter of each word.
    #[arg(long)]
    capitalize: bool,

    /// Append a random decimal digit to the passphrase.
    #[arg(long = "add-number")]
    add_number: bool,

    /// Append a random symbol to the passphrase.
    #[arg(long = "add-symbol")]
    add_symbol: bool,

    /// Emit N passphrases (one per line).
    #[arg(long, default_value_t = 1)]
    count: usize,

    /// Suppress the entropy line.
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Args, Debug)]
struct PwArgs {
    /// Password length in characters.
    #[arg(short, long, default_value_t = 20)]
    length: usize,

    /// Include uppercase letters (A-Z).
    #[arg(long)]
    upper: bool,

    /// Include lowercase letters (a-z).
    #[arg(long)]
    lower: bool,

    /// Include digits (0-9).
    #[arg(long)]
    digits: bool,

    /// Include symbols (printable ASCII punctuation).
    #[arg(long)]
    symbols: bool,

    /// Exclude visually ambiguous characters: 0 O o l 1 I |
    #[arg(long = "no-ambiguous")]
    no_ambiguous: bool,

    /// Emit N passwords (one per line).
    #[arg(long, default_value_t = 1)]
    count: usize,

    /// Suppress the entropy line.
    #[arg(short, long)]
    quiet: bool,
}

fn run_diceware(args: &DicewareArgs) -> Result<(), String> {
    if args.words == 0 {
        return Err("--words must be at least 1".to_string());
    }
    if args.count == 0 {
        return Err("--count must be at least 1".to_string());
    }

    let wl = wordlist::words();
    let opts = DicewareOptions {
        words: args.words,
        separator: args.sep.clone(),
        capitalize: args.capitalize,
        add_number: args.add_number,
        add_symbol: args.add_symbol,
    };

    let mut rng = OsRng;
    for _ in 0..args.count {
        let phrase = diceware::generate(&opts, &wl, &mut rng);
        println!("{phrase}");
    }

    if !args.quiet {
        let bits = entropy::diceware_bits(args.words, wl.len());
        // Note: --add-number / --add-symbol add a small amount of extra entropy
        // that is intentionally not counted here; the word entropy is the
        // conservative, honest floor.
        eprintln!(
            "entropy: {bits:.2} bits ({} words x log2({}))",
            args.words,
            wl.len()
        );
    }
    Ok(())
}

fn run_pw(args: &PwArgs) -> Result<(), String> {
    if args.count == 0 {
        return Err("--count must be at least 1".to_string());
    }

    // If the user names no class at all, fall back to a sensible default
    // (upper+lower+digits) rather than erroring — matches common expectations.
    let any_class = args.upper || args.lower || args.digits || args.symbols;
    let opts = CharsetOptions {
        length: args.length,
        upper: if any_class { args.upper } else { true },
        lower: if any_class { args.lower } else { true },
        digits: if any_class { args.digits } else { true },
        symbols: if any_class { args.symbols } else { false },
        no_ambiguous: args.no_ambiguous,
    };

    let mut rng = OsRng;
    let mut pool_size = 0usize;
    for _ in 0..args.count {
        let (pw, pool) = charset::generate(&opts, &mut rng).map_err(|e| e.to_string())?;
        pool_size = pool;
        println!("{pw}");
    }

    if !args.quiet {
        let bits = entropy::charset_bits(args.length, pool_size);
        eprintln!(
            "entropy: {bits:.2} bits ({} chars x log2({}))",
            args.length, pool_size
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match &cli.command {
        Some(Command::Pw(args)) => run_pw(args),
        None => run_diceware(&cli.diceware),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::FAILURE
        }
    }
}
