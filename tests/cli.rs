//! Integration tests driving the real `passphrase` binary via `assert_cmd`.
//!
//! These exercise the actual arg-parsing -> generate -> stdout/stderr path that
//! unit tests over the in-memory API cannot reach (flag marshaling, exit codes,
//! the entropy line on stderr, --quiet suppression).

use assert_cmd::Command;
use predicates::prelude::*;

fn bin() -> Command {
    Command::cargo_bin("passphrase").expect("binary `passphrase` should build")
}

// ---- diceware (default) mode ----

#[test]
fn default_emits_six_word_passphrase() {
    // Count with a `_` separator: EFF words contain only [a-z] and `-`, so
    // splitting on `_` reliably counts words even if a hyphenated word (e.g.
    // `t-shirt`) is drawn — splitting the default `-` separator would not.
    let out = bin()
        .args(["--words", "6", "--sep", "_"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let line = stdout.lines().next().unwrap();
    assert_eq!(line.split('_').count(), 6, "expected 6 words in `{line}`");
}

#[test]
fn separator_flag_is_used() {
    let out = bin()
        .args(["--words", "4", "--sep", "_"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let line = stdout.lines().next().unwrap();
    assert_eq!(
        line.matches('_').count(),
        3,
        "4 words => 3 underscores in `{line}`"
    );
}

#[test]
fn count_emits_n_lines() {
    let out = bin()
        .args(["--words", "5", "--count", "3"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout.lines().count(), 3, "expected 3 passphrase lines");
}

#[test]
fn entropy_line_printed_for_six_words() {
    // 6 words x log2(7776) ~= 77.55 bits -> printed to stderr.
    bin()
        .args(["--words", "6"])
        .assert()
        .success()
        .stderr(predicate::str::contains("77.55").and(predicate::str::contains("bits")));
}

#[test]
fn quiet_suppresses_entropy() {
    bin()
        .args(["--words", "6", "--quiet"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn capitalize_makes_each_word_initial_uppercase() {
    // `_` separator so each token is a whole word; --capitalize uppercases only
    // a word's first letter (not the half after an internal hyphen).
    let out = bin()
        .args(["--words", "5", "--sep", "_", "--capitalize", "--quiet"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let line = stdout.lines().next().unwrap();
    for word in line.split('_') {
        assert!(
            word.chars().next().unwrap().is_ascii_uppercase(),
            "`{word}` not capitalized"
        );
    }
}

#[test]
fn add_number_and_symbol_append_to_phrase() {
    let out = bin()
        .args(["--words", "4", "--add-number", "--add-symbol", "--quiet"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let line = stdout.lines().next().unwrap();
    let symbol_set = "!@#$%^&*()-_+=";
    let last = line.chars().last().unwrap();
    assert!(
        symbol_set.contains(last),
        "expected trailing symbol in `{line}`"
    );
}

#[test]
fn zero_words_is_an_error() {
    bin().args(["--words", "0"]).assert().failure();
}

// ---- pw (charset) mode ----

#[test]
fn pw_length_is_respected() {
    let out = bin()
        .args([
            "pw", "--length", "20", "--upper", "--lower", "--digits", "--quiet",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let line = stdout.lines().next().unwrap();
    assert_eq!(line.chars().count(), 20, "expected 20 chars in `{line}`");
}

#[test]
fn pw_no_ambiguous_excludes_the_set() {
    // Large length so the ambiguous chars would almost surely appear if present.
    let out = bin()
        .args([
            "pw",
            "--length",
            "400",
            "--upper",
            "--lower",
            "--digits",
            "--symbols",
            "--no-ambiguous",
            "--quiet",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let line = stdout.lines().next().unwrap();
    for amb in ['0', 'O', 'o', 'l', '1', 'I', '|'] {
        assert!(!line.contains(amb), "password contains ambiguous `{amb}`");
    }
}

#[test]
fn pw_only_digits_yields_only_digits() {
    let out = bin()
        .args(["pw", "--length", "30", "--digits", "--quiet"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let line = stdout.lines().next().unwrap();
    assert!(
        line.chars().all(|c| c.is_ascii_digit()),
        "`{line}` not all digits"
    );
}

#[test]
fn pw_entropy_line_for_full_pool() {
    // 20 chars x log2(94) ~= 131.09 bits.
    bin()
        .args([
            "pw",
            "--length",
            "20",
            "--upper",
            "--lower",
            "--digits",
            "--symbols",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("131.09").and(predicate::str::contains("bits")));
}

#[test]
fn pw_defaults_when_no_class_given() {
    // No class flags => fall back to upper+lower+digits, length 20.
    let out = bin().args(["pw", "--quiet"]).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let line = stdout.lines().next().unwrap();
    assert_eq!(line.chars().count(), 20);
    assert!(
        line.chars().all(|c| c.is_ascii_alphanumeric()),
        "`{line}` not alphanumeric"
    );
}

#[test]
fn pw_count_emits_n_lines() {
    let out = bin()
        .args(["pw", "--length", "12", "--lower", "--count", "4", "--quiet"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout.lines().count(), 4);
}

#[test]
fn help_succeeds() {
    bin()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("passphrase"));
}
