# passphrase

A secure, **fully offline** passphrase and password generator written in Rust.

- **Diceware mode (default):** memorable passphrases built from the bundled
  [EFF large wordlist](https://www.eff.org/dice) (7776 words).
- **Password mode (`pw`):** random-character passwords with per-class controls
  and an option to drop visually ambiguous characters.
- **Always prints an entropy estimate in bits**, so you know exactly how strong
  each secret is.

All randomness comes from the operating system CSPRNG (`rand::rngs::OsRng`,
backed by `getrandom`). The wordlist is embedded into the binary at compile time
with `include_str!`. **No network access, no API calls, no filesystem reads at
runtime.**

## Install / build

```sh
cargo build --release
# binary at target/release/passphrase
```

## Usage

### Diceware passphrase (default)

```sh
# six words joined by '-'
passphrase --words 6 --sep -
# -> correct-horse-battery-staple-anvil-rhubarb
# entropy: 77.55 bits (6 words x log2(7776))   [printed to stderr]

# five capitalized words, append a digit and a symbol
passphrase --words 5 --sep _ --capitalize --add-number --add-symbol

# generate 10 candidates at once
passphrase --words 6 --count 10

# suppress the entropy line (e.g. when piping)
passphrase --words 6 --quiet
```

Flags:

| Flag            | Meaning                                            | Default |
|-----------------|----------------------------------------------------|---------|
| `-w, --words N` | number of words                                    | `6`     |
| `--sep S`       | separator between words                            | `-`     |
| `--capitalize`  | capitalize the first letter of each word           | off     |
| `--add-number`  | append a random digit (0-9)                         | off     |
| `--add-symbol`  | append a random symbol                             | off     |
| `--count N`     | emit N passphrases (one per line)                  | `1`     |
| `-q, --quiet`   | suppress the entropy line                          | off     |

### Random-character password (`pw`)

```sh
passphrase pw --length 20 --upper --lower --digits --symbols --no-ambiguous
# -> a complex 20-char password with 0 O o l 1 I | excluded
# entropy: e.g. 128.81 bits (20 chars x log2(87))   [printed to stderr]
```

Flags:

| Flag             | Meaning                                              | Default |
|------------------|------------------------------------------------------|---------|
| `-l, --length N` | password length                                      | `20`    |
| `--upper`        | include uppercase `A-Z`                              | (see below) |
| `--lower`        | include lowercase `a-z`                              | (see below) |
| `--digits`       | include digits `0-9`                                | (see below) |
| `--symbols`      | include punctuation symbols                          | off     |
| `--no-ambiguous` | exclude `0 O o l 1 I \|`                             | off     |
| `--count N`      | emit N passwords (one per line)                      | `1`     |
| `-q, --quiet`    | suppress the entropy line                            | off     |

If you pass **no** class flag, `pw` defaults to upper + lower + digits.

## Entropy model

Entropy is the log2 of the size of the space the secret was drawn from, assuming
each selection is uniform and independent (which it is, because selections come
from a CSPRNG):

- **Diceware:** `words * log2(7776)`. Each EFF word contributes `log2(7776) ≈
  12.925` bits, so 6 words ≈ **77.55 bits**. (The optional appended
  digit/symbol from `--add-number`/`--add-symbol` adds a little more entropy
  that is deliberately *not* counted — the reported figure is a conservative
  floor.)
- **Random charset:** `length * log2(pool_size)`, where `pool_size` is the
  number of distinct characters in the enabled, post-`--no-ambiguous` pool.
  Full ASCII printable (94 chars) at length 20 ≈ **131.09 bits**.

## Architecture

A thin `main.rs` over a tested library:

- `src/wordlist.rs` — embeds and parses the EFF large wordlist.
- `src/diceware.rs` — passphrase generation over an **injected RNG**.
- `src/charset.rs`  — password generation + character-class pool building.
- `src/entropy.rs`  — entropy-in-bits math.

Every generator takes the RNG as `&mut impl Rng`. Production passes `OsRng`;
tests pass a seeded `StdRng` so output is deterministic and assertable. Run the
suite with `cargo test` (unit tests per module + `assert_cmd` integration tests
that drive the real binary).

## Wordlist attribution

This project bundles the **EFF Large Wordlist for Passphrases** (7776 words),
created by the Electronic Frontier Foundation (EFF). The list and the
methodology behind it are described at
<https://www.eff.org/deeplinks/2016/07/new-wordlists-random-passphrases> and the
list itself at <https://www.eff.org/dice>.

The EFF wordlists are released under the
[Creative Commons Attribution 3.0 United States (CC BY 3.0 US)](https://creativecommons.org/licenses/by/3.0/us/)
license. Thanks to Joseph Bonneau and the EFF for producing and publishing it.

## License

The `passphrase` source code is licensed under the [MIT License](LICENSE). The
bundled EFF wordlist is licensed separately under CC BY 3.0 US (see above).
