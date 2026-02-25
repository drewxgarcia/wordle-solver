use super::types::{PatternCode, Word};

pub const PATTERN_BUCKETS: usize = 3_usize.pow(5); // 243
pub const ALL_GREEN_CODE: PatternCode = 242;

#[inline]
fn li(b: u8) -> usize {
    assert!(b.is_ascii_lowercase());
    (b - b'a') as usize
}

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
pub(crate) fn simulate_results_code(guess: &Word, target: &Word) -> PatternCode {
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
    let mut code: PatternCode = 0;
    let mut pow: PatternCode = 1;
    for &digit in &state {
        code += (digit as PatternCode) * pow;
        pow *= 3;
    }
    code
}

pub fn parse_results_code(results: &str) -> Option<PatternCode> {
    let r = results.as_bytes();
    if r.len() != 5 {
        return None;
    }

    let mut code: PatternCode = 0;
    let mut pow: PatternCode = 1;
    for &b in r {
        let digit: PatternCode = match b {
            b'B' => 0,
            b'Y' => 1,
            b'G' => 2,
            _ => return None,
        };
        code += digit * pow;
        pow *= 3;
    }
    Some(code)
}

pub fn results_code_to_string(mut code: PatternCode) -> String {
    let mut out = [b'B'; 5];
    for slot in &mut out {
        let digit = code % 3;
        *slot = match digit {
            0 => b'B',
            1 => b'Y',
            2 => b'G',
            _ => unreachable!(),
        };
        code /= 3;
    }
    std::str::from_utf8(&out).unwrap().to_string()
}

pub fn simulate_results_pattern(guess: Word, target: Word) -> PatternCode {
    simulate_results_code(&guess, &target)
}
