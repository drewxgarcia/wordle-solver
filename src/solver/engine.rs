use super::load::load_word_list;
use super::oracle::{simulate_results_code, ALL_GREEN_CODE, PATTERN_BUCKETS};
use super::types::{GameStatus, PatternCode, SolverError, Word};
use rayon::prelude::*;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;

fn build_pattern_matrix(words: &[Word]) -> Vec<u8> {
    let n = words.len();
    if n == 0 {
        return Vec::new();
    }

    let mut matrix = vec![0u8; n * n];
    matrix
        .par_chunks_mut(n)
        .enumerate()
        .for_each(|(guess_idx, row)| {
            let guess = words[guess_idx];
            for (target_idx, target) in words.iter().enumerate() {
                row[target_idx] = simulate_results_code(&guess, target) as u8;
            }
        });
    matrix
}

fn calculate_entropy_row(row_start: usize, matrix: &[u8], candidates: &[usize]) -> f64 {
    let n = candidates.len();
    if n == 0 {
        return 0.0;
    }

    let mut buckets = [0u32; PATTERN_BUCKETS];
    for &target_idx in candidates {
        let code = matrix[row_start + target_idx] as usize;
        buckets[code] += 1;
    }

    let nf = n as f64;
    let mut h = 0.0;
    for &c in &buckets {
        if c == 0 {
            continue;
        }
        let p = (c as f64) / nf;
        h -= p * p.log2();
    }
    h
}

struct SolverCore {
    words: Vec<Word>,
    word_to_index: HashMap<Word, usize>,
    pattern_matrix: Vec<u8>, // row-major: guess_idx * word_count + target_idx
    word_count: usize,
}

#[derive(Clone)]
pub struct WordleSolver {
    core: Arc<SolverCore>,
    candidate_indices: Vec<usize>,
    candidate_words: Vec<Word>,
    attempts: usize,
    max_attempts: usize,
}

impl WordleSolver {
    pub(crate) fn from_word_list(word_list: Vec<Word>) -> Self {
        let word_count = word_list.len();
        let pattern_matrix = build_pattern_matrix(&word_list);

        let mut word_to_index = HashMap::with_capacity(word_count);
        for (idx, &word) in word_list.iter().enumerate() {
            word_to_index.entry(word).or_insert(idx);
        }

        let core = Arc::new(SolverCore {
            words: word_list,
            word_to_index,
            pattern_matrix,
            word_count,
        });

        Self {
            candidate_indices: (0..word_count).collect(),
            candidate_words: core.words.clone(),
            core,
            attempts: 0,
            max_attempts: 6,
        }
    }

    pub fn new(word_list_path: &str) -> io::Result<Self> {
        let words = load_word_list(word_list_path)?;
        Ok(Self::from_word_list(words))
    }

    pub fn candidates(&self) -> &[Word] {
        &self.candidate_words
    }

    pub fn attempt_number(&self) -> usize {
        self.attempts + 1
    }

    pub fn max_attempts(&self) -> usize {
        self.max_attempts
    }

    pub fn contains_word(&self, word: Word) -> bool {
        self.core.word_to_index.contains_key(&word)
    }

    pub fn scored_guesses(&self) -> Vec<(Word, f64)> {
        let candidates = &self.candidate_indices;
        let core = &self.core;
        let mut v: Vec<(Word, f64)> = self
            .candidate_indices
            .par_iter()
            .map(|&guess_idx| {
                let row_start = guess_idx * core.word_count;
                let h = calculate_entropy_row(row_start, &core.pattern_matrix, candidates);
                (core.words[guess_idx], h)
            })
            .collect();

        v.sort_by(|(wa, ha), (wb, hb)| hb.total_cmp(ha).then_with(|| wa.cmp(wb)));
        v
    }

    fn check_game_status(&self, code: PatternCode) -> GameStatus {
        if code == ALL_GREEN_CODE {
            GameStatus::Won
        } else if self.attempts >= self.max_attempts {
            GameStatus::Lost
        } else {
            GameStatus::Ongoing
        }
    }

    pub fn next_turn(
        &mut self,
        guess: Word,
        feedback: PatternCode,
    ) -> Result<GameStatus, SolverError> {
        if feedback as usize >= PATTERN_BUCKETS {
            return Err(SolverError::InvalidResultsPattern);
        }
        let code_u8 = feedback as u8;

        let Some(&guess_idx) = self.core.word_to_index.get(&guess) else {
            return Err(SolverError::GuessNotInWordList(guess));
        };
        let row_start = guess_idx * self.core.word_count;

        let next_candidate_indices: Vec<usize> = self
            .candidate_indices
            .iter()
            .copied()
            .filter(|&target_idx| self.core.pattern_matrix[row_start + target_idx] == code_u8)
            .collect();

        if next_candidate_indices.is_empty() {
            return Err(SolverError::InconsistentFeedback);
        }

        self.candidate_indices = next_candidate_indices;
        self.candidate_words = self
            .candidate_indices
            .iter()
            .map(|&idx| self.core.words[idx])
            .collect();
        self.attempts += 1;
        Ok(self.check_game_status(feedback))
    }
}
