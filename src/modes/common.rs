use crate::solver::Word;
use crate::ui::{clear_screen, read_line_trimmed};

pub type CommandSpec = (&'static str, &'static str);

pub const GAME_MODE_TITLE: &str = "Game Mode";
pub const GAME_MODE_SUBTITLE: &str = "Enter a 5-letter guess";
pub const GAME_COMMANDS: &[CommandSpec] = &[
    ("/HELP", "Show this command list"),
    ("/HINT [n]", "Show top n suggested guesses (default 5)"),
    ("/STATUS", "Show turn/candidate status"),
    ("/BOARD", "Reprint the board"),
    ("/UNDO", "Undo the previous accepted guess"),
    ("/EXIT", "Return to main menu"),
];

pub const SOLVER_MODE_TITLE: &str = "Solver Mode";
pub const SOLVER_MODE_SUBTITLE: &str = "Enter feedback as 5 chars: G/Y/B (example: GYBBY)";
pub const SOLVER_COMMANDS: &[CommandSpec] = &[
    ("HELP", "Show this command list"),
    ("STATUS", "Show turn and candidate status"),
    ("TOP [n]", "Show top n suggestions (default 10)"),
    (
        "CANDS [n]",
        "Show first n remaining candidates (default 10)",
    ),
    ("BOARD", "Show guess history with colored feedback"),
    ("UNDO", "Revert the previous accepted turn"),
    ("EXIT", "Return to main menu"),
];

pub fn wait_for_enter(message: &str) {
    println!("{message}");
    let _ = read_line_trimmed();
}

pub fn read_mode_line() -> Option<String> {
    match read_line_trimmed() {
        Ok(Some(input)) => Some(input),
        Ok(None) => {
            println!("EOF received. Exiting mode.");
            None
        }
        Err(e) => {
            eprintln!("Failed to read input: {e}");
            None
        }
    }
}

pub fn print_mode_header(title: &str, subtitle: &str, commands: &str) {
    clear_screen();
    println!(
        "========================\n\
         {title}\n\
         {subtitle}\n\
         Commands: {commands}\n\
         ========================\n"
    );
}

pub fn command_summary(commands: &[CommandSpec]) -> String {
    commands
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn print_commands(commands: &[CommandSpec]) {
    println!("Commands:");
    for (name, description) in commands {
        println!("  {:<12} {}", name, description);
    }
}

pub fn print_top_words(scored: &[(Word, f64)], n: usize, suffix: &str, empty_message: &str) {
    let limit = n.min(scored.len());
    if limit == 0 {
        println!("{empty_message}");
        return;
    }
    println!("Top {limit} {suffix}:");
    for (i, (w, _)) in scored.iter().take(limit).enumerate() {
        println!("  {:>2}. {}", i + 1, w);
    }
}

pub fn print_first_words(words: &[Word], n: usize, suffix: &str, empty_message: &str) {
    let limit = n.min(words.len());
    if limit == 0 {
        println!("{empty_message}");
        return;
    }
    println!("First {limit} {suffix}:");
    for (i, w) in words.iter().take(limit).enumerate() {
        println!("  {:>2}. {}", i + 1, w);
    }
}

pub fn print_undo_result(undid: bool) {
    if undid {
        println!("Previous turn undone.");
    } else {
        println!("Nothing to undo yet.");
    }
}

pub fn print_status(
    turn: usize,
    max_turns: usize,
    count_label: &str,
    count: usize,
    current_guess: Option<Word>,
) {
    if let Some(guess) = current_guess {
        println!("Status: turn {turn}/{max_turns} | {count_label} {count} | current guess {guess}");
    } else {
        println!("Status: turn {turn}/{max_turns} | {count_label} {count}");
    }
}
