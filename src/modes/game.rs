use crate::modes::common::{
    command_summary, print_commands, print_mode_header, print_status, print_top_words,
    print_undo_result, read_mode_line, wait_for_enter, GAME_COMMANDS, GAME_MODE_SUBTITLE,
    GAME_MODE_TITLE,
};
use crate::session::GameSession;
use crate::solver::{simulate_results_pattern, GameStatus, Word};
use crate::ui::{clear_screen, parse_game_decision, render_board, GameDecision};
use rand::seq::SliceRandom;
use std::io;

fn pick_secret(words: &[Word]) -> Option<Word> {
    let mut rng = rand::thread_rng();
    words.choose(&mut rng).copied()
}

pub fn run(wordlist_path: &str) -> io::Result<()> {
    let mut session = GameSession::new(wordlist_path)?;
    let all_words = session.solver().candidates().to_vec();
    let command_line = command_summary(GAME_COMMANDS);

    let Some(secret) = pick_secret(&all_words) else {
        println!("Word list is empty. Cannot start a game.");
        wait_for_enter("Press Enter to return to the main menu.");
        return Ok(());
    };

    'game: loop {
        print_mode_header(GAME_MODE_TITLE, GAME_MODE_SUBTITLE, &command_line);

        render_board(session.turns());
        println!(
            "Turn {}/{}",
            session.solver().attempt_number(),
            session.solver().max_attempts()
        );
        println!(
            "Possible answers remaining: {}",
            session.solver().candidates().len()
        );
        println!("Enter guess or command:");

        let Some(input) = read_mode_line() else {
            break 'game;
        };

        let guess = match parse_game_decision(&input) {
            GameDecision::SubmitGuess(guess) => guess,
            GameDecision::Help => {
                print_commands(GAME_COMMANDS);
                wait_for_enter("Press Enter to continue.");
                continue;
            }
            GameDecision::Hints(n) => {
                let scored = session.solver().scored_guesses();
                print_top_words(&scored, n, "suggestions", "No suggestions available.");
                wait_for_enter("Press Enter to continue.");
                continue;
            }
            GameDecision::Status => {
                print_status(
                    session.solver().attempt_number(),
                    session.solver().max_attempts(),
                    "possible answers",
                    session.solver().candidates().len(),
                    None,
                );
                wait_for_enter("Press Enter to continue.");
                continue;
            }
            GameDecision::Board => {
                render_board(session.turns());
                wait_for_enter("Press Enter to continue.");
                continue;
            }
            GameDecision::Undo => {
                print_undo_result(session.undo());
                wait_for_enter("Press Enter to continue.");
                continue;
            }
            GameDecision::ExitMode => return Ok(()),
            GameDecision::UnknownCommand => {
                println!("Unknown command. Use /HELP.");
                wait_for_enter("Press Enter to continue.");
                continue;
            }
            GameDecision::InvalidGuess => {
                println!("Invalid guess. Enter a 5-letter word or a command like /HELP.");
                wait_for_enter("Press Enter to continue.");
                continue;
            }
        };

        if !session.solver().contains_word(guess) {
            println!("'{guess}' is not in the allowed word list.");
            wait_for_enter("Press Enter to continue.");
            continue;
        }

        let results_code = simulate_results_pattern(guess, secret);
        let status = match session.apply_turn(guess, results_code) {
            Ok(status) => status,
            Err(e) => {
                println!("Internal error: {e}");
                wait_for_enter("Press Enter to continue.");
                continue;
            }
        };

        match status {
            GameStatus::Won => {
                clear_screen();
                render_board(session.turns());
                println!("You solved it in {} turns!", session.turns().len());
                println!("Answer: {secret}");
                wait_for_enter("Press Enter to return to the main menu.");
                break;
            }
            GameStatus::Lost => {
                clear_screen();
                render_board(session.turns());
                println!("Out of turns.");
                println!("Answer: {secret}");
                wait_for_enter("Press Enter to return to the main menu.");
                break;
            }
            GameStatus::Ongoing => {}
        }
    }

    Ok(())
}
