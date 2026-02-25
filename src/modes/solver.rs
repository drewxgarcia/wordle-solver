use crate::modes::common::{
    command_summary, print_commands, print_first_words, print_mode_header, print_status,
    print_top_words, print_undo_result, read_mode_line, wait_for_enter, SOLVER_COMMANDS,
    SOLVER_MODE_SUBTITLE, SOLVER_MODE_TITLE,
};
use crate::session::GameSession;
use crate::solver::{GameStatus, Word};
use crate::ui::{
    parse_solver_choice, parse_solver_feedback, render_board, tie_set, SolverChoiceDecision,
    SolverFeedbackDecision,
};
use std::io;

pub fn run(wordlist_path: &str) -> io::Result<()> {
    let mut session = GameSession::new(wordlist_path)?;
    let command_line = command_summary(SOLVER_COMMANDS);

    'game: loop {
        print_mode_header(SOLVER_MODE_TITLE, SOLVER_MODE_SUBTITLE, &command_line);

        render_board(session.turns());
        let remaining = session.solver().candidates().len();
        println!(
            "Turn {}/{} | Remaining candidates: {}",
            session.solver().attempt_number(),
            session.solver().max_attempts(),
            remaining
        );

        let scored = session.solver().scored_guesses();
        if scored.is_empty() {
            println!("No candidates remain. Check for inconsistent feedback.");
            break;
        }

        let ties = tie_set(&scored, 1e-10);
        let guess: Word = if ties.len() > 1 {
            println!("Multiple top-ranked guesses ({} total):", ties.len());
            for (i, (w, _)) in ties.iter().take(10).enumerate() {
                println!("   {}. {}", i + 1, w);
            }
            if ties.len() > 10 {
                println!("   ... +{} more", ties.len() - 10);
            }
            println!("Pick a guess (number or word), or press Enter for #1.");
            loop {
                let Some(choice) = read_mode_line() else {
                    break 'game;
                };

                match parse_solver_choice(&choice, ties) {
                    SolverChoiceDecision::Pick(picked) => break picked,
                    SolverChoiceDecision::InvalidChoice => {
                        println!("Invalid choice. Enter a valid number or one of the tied words:");
                    }
                }
            }
        } else {
            println!("Suggested guess: {}", scored[0].0);
            scored[0].0
        };

        println!("Enter results for '{guess}': ");
        let results_code = loop {
            let Some(input) = read_mode_line() else {
                break 'game;
            };
            match parse_solver_feedback(&input) {
                SolverFeedbackDecision::SubmitFeedback(code) => break code,
                SolverFeedbackDecision::Help => {
                    print_commands(SOLVER_COMMANDS);
                    println!("You can also enter a 5-letter G/Y/B pattern directly.");
                }
                SolverFeedbackDecision::Status => {
                    print_status(
                        session.solver().attempt_number(),
                        session.solver().max_attempts(),
                        "candidates",
                        session.solver().candidates().len(),
                        Some(guess),
                    );
                }
                SolverFeedbackDecision::Top(n) => {
                    print_top_words(&scored, n, "ranked guesses", "No scored guesses available.");
                }
                SolverFeedbackDecision::Cands(n) => {
                    let cands = session.solver().candidates();
                    print_first_words(cands, n, "candidates", "No candidates remain.");
                }
                SolverFeedbackDecision::Board => {
                    render_board(session.turns());
                }
                SolverFeedbackDecision::InvalidInput => {
                    println!("Invalid input. Enter G/Y/B, or type HELP.");
                }
                SolverFeedbackDecision::UndoRestart => {
                    print_undo_result(session.undo());
                    println!();
                    continue 'game;
                }
                SolverFeedbackDecision::ExitMode => return Ok(()),
            }
        };

        let status = match session.apply_turn(guess, results_code) {
            Ok(status) => status,
            Err(e) => {
                println!("{e}");
                println!("State unchanged. Please re-enter feedback for the same guess.");
                continue;
            }
        };

        match status {
            GameStatus::Won => {
                println!("Congratulations, you won!");
                break;
            }
            GameStatus::Lost => {
                println!("Game over. Better luck next time!");
                break;
            }
            GameStatus::Ongoing => {}
        }

        println!();
    }

    wait_for_enter("Press Enter to return to the main menu.");
    Ok(())
}
