use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Word([u8; 5]);

impl Word {
    #[inline]
    fn as_bytes(&self) -> &[u8; 5] {
        &self.0
    }
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Safe because words are normalized to ASCII lowercase at load time.
        let s = std::str::from_utf8(&self.0).unwrap();
        write!(f, "{s}")
    }
}

#[inline]
fn li(b: u8) -> usize {
    assert!(b.is_ascii_lowercase());
    (b - b'a') as usize
}

#[inline]
fn bit(pos: usize) -> u8 {
    1u8 << pos
}

const PATTERN_BUCKETS: usize = 3_usize.pow(5); // 243

/// Parse a trimmed line into Word([u8; 5]).
/// Accepts ASCII letters only, lowercases A-Z, rejects anything else.
pub fn parse_word5(line: &str) -> Option<Word> {
    let s = line.trim().as_bytes();
    if s.len() != 5 {
        return None;
    }

    let mut w = [0u8; 5];
    for i in 0..5 {
        let b = s[i];
        let lower = match b {
            b'a'..=b'z' => b,
            b'A'..=b'Z' => b + 32,
            _ => return None,
        };
        w[i] = lower;
    }
    Some(Word(w))
}

/// Wordle-correct feedback simulation with duplicates.
/// Encodes pattern into a base-3 code in [0..243):
/// B=0, Y=1, G=2 at each position.
#[inline]
fn simulate_results_code(guess: &Word, target: &Word) -> u16 {
    let g = guess.as_bytes();
    let t = target.as_bytes();

    let mut counts = [0u8; 26];
    for &c in t.iter() {
        counts[li(c)] += 1;
    }

    // state: 0=B, 1=Y, 2=G
    let mut state = [0u8; 5];

    // Pass 1: greens
    for i in 0..5 {
        if g[i] == t[i] {
            state[i] = 2;
            let idx = li(g[i]);
            counts[idx] -= 1;
        }
    }

    // Pass 2: yellows
    for i in 0..5 {
        if state[i] == 0 {
            let idx = li(g[i]);
            if counts[idx] > 0 {
                state[i] = 1;
                counts[idx] -= 1;
            }
        }
    }

    // Pack base-3.
    let mut code: u16 = 0;
    let mut pow: u16 = 1;
    for i in 0..5 {
        code += (state[i] as u16) * pow;
        pow *= 3;
    }
    code
}

fn calculate_entropy(guess: &Word, candidates: &[Word]) -> f64 {
    let n = candidates.len();
    if n == 0 {
        return 0.0;
    }

    let mut buckets = [0u32; PATTERN_BUCKETS];

    for t in candidates.iter() {
        let code = simulate_results_code(guess, t) as usize;
        buckets[code] += 1;
    }

    let nf = n as f64;
    let mut h = 0.0;
    for &c in buckets.iter() {
        if c == 0 {
            continue;
        }
        let p = (c as f64) / nf;
        h -= p * p.log2();
    }
    h
}

#[derive(Clone)]
pub struct WordleSolver {
    // Current candidate list.
    word_list: Vec<Word>,

    // Constraints (all indexed by 0..26 for a..z).
    greens: [Option<u8>; 5], // position -> letter index
    banned: [u8; 26],        // letter -> bitmask of banned positions
    min_counts: [u8; 26],    // letter -> minimum occurrences
    max_counts: [u8; 26],    // letter -> maximum occurrences (0 means absent)

    attempts: usize,
    max_attempts: usize,
}

impl WordleSolver {
    pub fn new(word_list_path: &str) -> io::Result<Self> {
        let file = File::open(word_list_path)?;
        let reader = BufReader::new(file);

        let word_list: Vec<Word> = reader
            .lines()
            .filter_map(|line| parse_word5(&line.ok()?))
            .collect();

        Ok(Self {
            word_list,
            greens: [None; 5],
            banned: [0; 26],
            min_counts: [0; 26],
            max_counts: [5; 26], // at most 5 occurrences of any letter
            attempts: 0,
            max_attempts: 6,
        })
    }

    pub fn candidates(&self) -> &[Word] {
        &self.word_list
    }

    pub fn attempt_number(&self) -> usize {
        self.attempts + 1
    }

    pub fn max_attempts(&self) -> usize {
        self.max_attempts
    }

    pub fn scored_guesses(&self) -> Vec<(Word, f64)> {
        let mut v: Vec<(Word, f64)> = self
            .word_list
            .par_iter()
            .map(|w| (*w, calculate_entropy(w, &self.word_list)))
            .collect();

        v.sort_by(|(wa, ha), (wb, hb)| hb.total_cmp(ha).then_with(|| wa.cmp(wb)));
        v
    }

    fn validate_constraints(
        greens: &[Option<u8>; 5],
        banned: &[u8; 26],
        min_counts: &[u8; 26],
        max_counts: &[u8; 26],
    ) -> Result<(), String> {
        for i in 0..26 {
            if min_counts[i] > max_counts[i] {
                return Err(format!(
                    "Inconsistent feedback: '{}' requires at least {} but at most {}",
                    (b'a' + i as u8) as char,
                    min_counts[i],
                    max_counts[i]
                ));
            }
        }

        // Green can't be banned.
        for pos in 0..5 {
            if let Some(letter) = greens[pos] {
                let i = letter as usize;
                if (banned[i] & bit(pos)) != 0 {
                    return Err(format!(
                        "Inconsistent feedback: position {} is green '{}' but also banned",
                        pos,
                        (b'a' + i as u8) as char
                    ));
                }
            }
        }
        Ok(())
    }

    fn process_results(&mut self, guess: Word, results: &str) -> Result<(), String> {
        if results.len() != 5 {
            return Err("Invalid results length: expected 5 characters".to_string());
        }
        let r = results.as_bytes();
        if !r.iter().all(|b| matches!(b, b'G' | b'Y' | b'B')) {
            return Err("Invalid results characters: use only G, Y, and B".to_string());
        }

        let g = guess.as_bytes();

        let mut next_greens = self.greens;
        let mut next_banned = self.banned;
        let mut next_min_counts = self.min_counts;
        let mut next_max_counts = self.max_counts;

        let mut guess_total = [0u8; 26];
        let mut gy_total = [0u8; 26];
        let mut b_total = [0u8; 26];

        for pos in 0..5 {
            let idx = li(g[pos]);
            guess_total[idx] += 1;

            match r[pos] {
                b'G' => {
                    // If we previously banned this exact letter at this position (from earlier Y/B),
                    // green overrides it: clear it.
                    next_banned[idx] &= !bit(pos);

                    // Don't silently overwrite an existing green with a different letter.
                    match next_greens[pos] {
                        None => next_greens[pos] = Some(idx as u8),
                        Some(prev) => {
                            if prev as usize != idx {
                                return Err(format!(
                                    "Inconsistent feedback: pos {} was green '{}' now '{}'",
                                    pos,
                                    (b'a' + prev) as char,
                                    g[pos] as char
                                ));
                            }
                        }
                    }

                    gy_total[idx] += 1;
                }
                b'Y' => {
                    next_banned[idx] |= bit(pos);
                    gy_total[idx] += 1;
                }
                b'B' => {
                    next_banned[idx] |= bit(pos);
                    b_total[idx] += 1;
                }
                _ => unreachable!(),
            }
        }

        // Update min/max bounds from this guess.
        for i in 0..26 {
            let gy = gy_total[i];
            let b = b_total[i];

            if gy > 0 {
                next_min_counts[i] = next_min_counts[i].max(gy);
            }

            if gy == 0 && b > 0 {
                next_max_counts[i] = 0;
            } else if gy > 0 && b > 0 && guess_total[i] == gy + b {
                next_max_counts[i] = next_max_counts[i].min(gy);
            }
        }

        Self::validate_constraints(
            &next_greens,
            &next_banned,
            &next_min_counts,
            &next_max_counts,
        )?;

        self.greens = next_greens;
        self.banned = next_banned;
        self.min_counts = next_min_counts;
        self.max_counts = next_max_counts;

        self.word_list = self
            .word_list
            .iter()
            .copied()
            .filter(|w| self.is_possible_word(*w))
            .collect();
        Ok(())
    }

    fn is_possible_word(&self, word: Word) -> bool {
        let w = word.as_bytes();

        // Greens + banned positions.
        for pos in 0..5 {
            let idx = li(w[pos]);

            if let Some(g) = self.greens[pos] {
                if idx != g as usize {
                    return false;
                }
            }
            if (self.banned[idx] & bit(pos)) != 0 {
                return false;
            }
        }

        // Counts.
        let mut counts = [0u8; 26];
        for &b in w.iter() {
            counts[li(b)] += 1;
        }

        // Min/max bounds.
        for i in 0..26 {
            if counts[i] < self.min_counts[i] {
                return false;
            }
            if counts[i] > self.max_counts[i] {
                return false;
            }
        }

        true
    }

    fn check_game_status(&self, results: &str) -> &'static str {
        let r = results.as_bytes();
        if r.iter().all(|&b| b == b'G') {
            "won"
        } else if self.attempts >= self.max_attempts {
            "lost"
        } else {
            "ongoing"
        }
    }

    pub fn next_turn(&mut self, guess: Word, results: &str) -> Result<&'static str, String> {
        self.process_results(guess, results)?;
        self.attempts += 1;
        Ok(self.check_game_status(results))
    }
}
