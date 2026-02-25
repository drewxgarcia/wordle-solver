use super::engine::WordleSolver;
use super::load::load_word_list;
use super::oracle::{
    parse_results_code, parse_word5, results_code_to_string, simulate_results_code,
    simulate_results_pattern,
};
use super::types::{GameStatus, SolverError, Word};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn word(s: &str) -> Word {
    parse_word5(s).unwrap()
}

#[test]
fn duplicate_letters_match_wordle_oracle() {
    let guess = word("allee");
    let target = word("apple");
    let code = simulate_results_code(&guess, &target);
    assert_eq!(code, parse_results_code("GYBBG").unwrap());
}

#[test]
fn pattern_code_round_trip() {
    for pattern in ["BBBBB", "GYBBG", "YYYYY", "GGGGG"] {
        let code = parse_results_code(pattern).unwrap();
        assert_eq!(results_code_to_string(code), pattern);
    }
}

#[test]
fn next_turn_filters_by_oracle_code() {
    let words = vec![word("arise"), word("raise"), word("serai"), word("irate")];
    let mut solver = WordleSolver::from_word_list(words);

    let status = solver
        .next_turn(word("raise"), parse_results_code("YYGGG").unwrap())
        .unwrap();
    assert_eq!(status, GameStatus::Ongoing);
    assert_eq!(solver.candidates(), &[word("arise")]);
}

#[test]
fn next_turn_accepts_guess_not_in_current_candidates() {
    let words = vec![word("arise"), word("raise"), word("serai"), word("irate")];
    let mut solver = WordleSolver::from_word_list(words);

    solver
        .next_turn(word("raise"), parse_results_code("YYGGG").unwrap())
        .unwrap();
    let feedback = simulate_results_pattern(word("serai"), word("arise"));
    let status = solver.next_turn(word("serai"), feedback).unwrap();
    assert_eq!(status, GameStatus::Ongoing);
    assert_eq!(solver.candidates(), &[word("arise")]);
}

#[test]
fn inconsistent_feedback_does_not_mutate_state() {
    let words = vec![word("arise"), word("raise"), word("serai"), word("irate")];
    let mut solver = WordleSolver::from_word_list(words);
    let before = solver.candidates().to_vec();

    let err = solver
        .next_turn(word("raise"), parse_results_code("BBBBB").unwrap())
        .unwrap_err();
    assert_eq!(err, SolverError::InconsistentFeedback);
    assert_eq!(solver.candidates(), before.as_slice());
    assert_eq!(solver.attempt_number(), 1);
}

fn write_temp_wordlist(contents: &str) -> String {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("wordle_solver_test_{stamp}.txt"));
    fs::write(&path, contents).unwrap();
    path.to_string_lossy().to_string()
}

#[test]
fn load_word_list_deduplicates_while_preserving_order() {
    let path = write_temp_wordlist("apple\nberry\nAPPLE\nchase\n");
    let loaded = load_word_list(&path).unwrap();
    let _ = fs::remove_file(&path);

    assert_eq!(loaded, vec![word("apple"), word("berry"), word("chase")]);
}

#[test]
fn load_word_list_rejects_invalid_lines() {
    let path = write_temp_wordlist("apple\nbad!\nchase\n");
    let err = load_word_list(&path).unwrap_err();
    let _ = fs::remove_file(&path);

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(
        err.to_string().contains("line 2"),
        "expected line number in error, got: {err}"
    );
}
