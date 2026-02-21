use crate::solver::{parse_word5, Word};
use std::io::{self, Write};

pub enum PromptCommand {
    Help,
    Status,
    Top(usize),
    Cands(usize),
    Board,
    Undo,
    Exit,
}

pub enum PromptInput {
    Results(String),
    Command(PromptCommand),
}

pub fn valid_results(results: &str) -> bool {
    let b = results.as_bytes();
    b.len() == 5 && b.iter().all(|&c| matches!(c, b'G' | b'Y' | b'B'))
}

pub fn tie_set(scored: &[(Word, f64)], eps: f64) -> &[(Word, f64)] {
    let best = scored[0].1;
    let mut end = 1usize;
    while end < scored.len() && (scored[end].1 - best).abs() <= eps {
        end += 1;
    }
    &scored[..end]
}

pub fn read_line_trimmed() -> String {
    let mut s = String::new();
    io::stdin().read_line(&mut s).expect("read line failed");
    s.trim().to_string()
}

pub fn parse_choice(input: &str, scored: &[(Word, f64)]) -> Option<Word> {
    if input.is_empty() {
        return Some(scored[0].0);
    }
    // Allow selecting by rank number.
    if let Ok(k) = input.parse::<usize>() {
        if (1..=scored.len()).contains(&k) {
            return Some(scored[k - 1].0);
        }
    }
    // Allow typing the word itself.
    if let Some(w) = parse_word5(input) {
        if scored.iter().any(|(x, _)| *x == w) {
            return Some(w);
        }
    }
    None
}

pub fn parse_prompt_input(raw: &str) -> Option<PromptInput> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let upper = trimmed.to_ascii_uppercase();
    if valid_results(&upper) {
        return Some(PromptInput::Results(upper));
    }

    let mut parts = upper.split_whitespace();
    let cmd = parts.next()?;
    let arg = parts.next();
    let parsed_n = arg.and_then(|s| s.parse::<usize>().ok()).unwrap_or(10);

    let command = match cmd {
        "HELP" => PromptCommand::Help,
        "STATUS" => PromptCommand::Status,
        "TOP" => PromptCommand::Top(parsed_n.max(1)),
        "CANDS" => PromptCommand::Cands(parsed_n.max(1)),
        "BOARD" => PromptCommand::Board,
        "UNDO" => PromptCommand::Undo,
        "EXIT" => PromptCommand::Exit,
        _ => return None,
    };
    Some(PromptInput::Command(command))
}

pub fn render_board(board: &[(Word, String)]) {
    if board.is_empty() {
        println!("Board: (empty)");
        return;
    }

    println!("Board:");
    for (i, (guess, results)) in board.iter().enumerate() {
        let tiles: String = results
            .chars()
            .map(|c| match c {
                'G' => "🟩",
                'Y' => "🟨",
                _ => "⬛",
            })
            .collect();
        println!("  {:>2}. {}  {}", i + 1, guess, tiles);
    }
}

pub fn clear_screen() {
    // ANSI clear-screen + cursor-home. Works in modern terminals on Windows/macOS/Linux.
    print!("\x1B[2J\x1B[H");
    let _ = io::stdout().flush();
}
