# Drew's Wordle Solver

A fast, interactive terminal Wordle solver built in Rust.

It recommends high-information guesses, tracks constraints exactly (including duplicate-letter edge cases), and provides a command-driven UX.

## Highlights

- Wordle-correct feedback handling (`G`, `Y`, `B`), including duplicates.
- Entropy-based guess ranking for strong information gain each turn.
- Interactive terminal dashboard with:
  - Emoji board history (`🟩 🟨 ⬛`)
  - `UNDO` support
  - `HELP`, `STATUS`, `TOP [n]`, `CANDS [n]`, `BOARD`, `EXIT`
- Bundled word list (`wordlist.txt`) for batteries-included usage.
- Clean architecture split between solver logic and UI logic.

## Quick Start

### Requirements

- Rust toolchain (stable)

### Run

```bash
cargo run
```

### Build release binary

```bash
cargo build --release
```

Then run:

```bash
./target/release/solver_project
```

On Windows:

```powershell
.\target\release\solver_project.exe
```

## How To Use

Each turn, the solver suggests a guess. Enter Wordle feedback as a 5-character pattern:

- `G` = green (right letter, right position)
- `Y` = yellow (right letter, wrong position)
- `B` = black/gray (letter not used in that position)

Example:

- Guess: `crane`
- Feedback input: `BYGBB`

### Commands

- `HELP`: show command help
- `STATUS`: show turn and candidate count
- `TOP [n]`: show top ranked guesses
- `CANDS [n]`: show first `n` remaining candidates
- `BOARD`: show guess history with colored squares
- `UNDO`: revert previous accepted turn
- `EXIT`: quit

## Methodology: Entropy-Based Guess Selection

The solver uses **expected information gain** to rank guesses.

For a candidate set of possible answers `S` and a guess `g`:

1. Simulate Wordle feedback pattern for `g` against every target in `S`.
2. Group targets by resulting feedback pattern (there are `3^5 = 243` possible patterns).
3. Convert bucket counts to probabilities `p_i`.
4. Compute entropy:

`H(g) = -Σ p_i log2(p_i)`

The guess with highest entropy is ranked first because it is expected to split the remaining search space most effectively.

### Why this works well

- A “good” guess is one that maximizes discrimination across remaining candidates.
- Entropy naturally rewards guesses that produce many distinct, balanced feedback partitions.

## Wordle Correctness Details

Feedback simulation follows Wordle’s two-pass logic:

1. **Greens first**: exact-position matches consume letter counts.
2. **Yellows second**: non-green letters are marked yellow only if remaining count for that letter is still available.

This is critical for duplicate letters and avoids common incorrect implementations.