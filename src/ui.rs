use crate::session::Turn;
use crate::solver::{parse_results_code, parse_word5, results_code_to_string, PatternCode, Word};
use std::io::{self, Write};

pub enum GameDecision {
    SubmitGuess(Word),
    Help,
    Hints(usize),
    Status,
    Board,
    Undo,
    ExitMode,
    UnknownCommand,
    InvalidGuess,
}

pub enum SolverChoiceDecision {
    Pick(Word),
    InvalidChoice,
}

pub enum SolverFeedbackDecision {
    SubmitFeedback(PatternCode),
    Help,
    Status,
    Top(usize),
    Cands(usize),
    Board,
    UndoRestart,
    ExitMode,
    InvalidInput,
}

pub fn tie_set(scored: &[(Word, f64)], eps: f64) -> &[(Word, f64)] {
    let best = scored[0].1;
    let mut end = 1usize;
    while end < scored.len() && (scored[end].1 - best).abs() <= eps {
        end += 1;
    }
    &scored[..end]
}

pub fn read_line_trimmed() -> io::Result<Option<String>> {
    let mut s = String::new();
    let n = io::stdin().read_line(&mut s)?;
    if n == 0 {
        return Ok(None);
    }
    Ok(Some(s.trim().to_string()))
}

pub fn parse_solver_choice(input: &str, scored: &[(Word, f64)]) -> SolverChoiceDecision {
    if input.is_empty() {
        return SolverChoiceDecision::Pick(scored[0].0);
    }
    // Allow selecting by rank number.
    if let Ok(k) = input.parse::<usize>() {
        if (1..=scored.len()).contains(&k) {
            return SolverChoiceDecision::Pick(scored[k - 1].0);
        }
    }
    // Allow typing the word itself.
    if let Some(w) = parse_word5(input) {
        if scored.iter().any(|(x, _)| *x == w) {
            return SolverChoiceDecision::Pick(w);
        }
    }
    SolverChoiceDecision::InvalidChoice
}

pub fn parse_solver_feedback(raw: &str) -> SolverFeedbackDecision {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return SolverFeedbackDecision::InvalidInput;
    }

    let upper = trimmed.to_ascii_uppercase();
    if let Some(code) = parse_results_code(&upper) {
        return SolverFeedbackDecision::SubmitFeedback(code);
    }

    let mut parts = upper.split_whitespace();
    let Some(cmd) = parts.next() else {
        return SolverFeedbackDecision::InvalidInput;
    };
    let arg = parts.next();
    let parsed_n = arg.and_then(|s| s.parse::<usize>().ok()).unwrap_or(10);

    match cmd {
        "HELP" => SolverFeedbackDecision::Help,
        "STATUS" => SolverFeedbackDecision::Status,
        "TOP" => SolverFeedbackDecision::Top(parsed_n.max(1)),
        "CANDS" => SolverFeedbackDecision::Cands(parsed_n.max(1)),
        "BOARD" => SolverFeedbackDecision::Board,
        "UNDO" => SolverFeedbackDecision::UndoRestart,
        "EXIT" => SolverFeedbackDecision::ExitMode,
        _ => SolverFeedbackDecision::InvalidInput,
    }
}

pub fn parse_game_decision(raw: &str) -> GameDecision {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return GameDecision::InvalidGuess;
    }

    if let Some(cmd_text) = trimmed.strip_prefix('/') {
        let upper = cmd_text.trim().to_ascii_uppercase();
        let mut parts = upper.split_whitespace();
        let Some(cmd) = parts.next() else {
            return GameDecision::UnknownCommand;
        };
        let arg = parts.next();
        let parsed_n = arg.and_then(|s| s.parse::<usize>().ok()).unwrap_or(5);

        return match cmd {
            "HELP" => GameDecision::Help,
            "HINT" => GameDecision::Hints(parsed_n.max(1)),
            "STATUS" => GameDecision::Status,
            "BOARD" => GameDecision::Board,
            "UNDO" => GameDecision::Undo,
            "EXIT" => GameDecision::ExitMode,
            _ => GameDecision::UnknownCommand,
        };
    }

    parse_word5(trimmed)
        .map(GameDecision::SubmitGuess)
        .unwrap_or(GameDecision::InvalidGuess)
}

pub fn render_board(turns: &[Turn]) {
    if turns.is_empty() {
        println!("Board: (empty)");
        return;
    }

    println!("Board:");
    for (i, turn) in turns.iter().enumerate() {
        let results = results_code_to_string(turn.feedback);
        let tiles: String = results
            .chars()
            .map(|c| match c {
                'G' => "🟩",
                'Y' => "🟨",
                _ => "⬛",
            })
            .collect();
        println!("  {:>2}. {}  {}", i + 1, turn.guess, tiles);
    }
}

pub fn clear_screen() {
    // ANSI clear-screen + cursor-home. Works in modern terminals on Windows/macOS/Linux.
    print!("\x1B[2J\x1B[H");
    let _ = io::stdout().flush();
}
