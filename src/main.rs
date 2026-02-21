mod solver;
mod ui;

use solver::WordleSolver;
use std::io;
use ui::{
    clear_screen, parse_choice, parse_prompt_input, read_line_trimmed, render_board, tie_set,
    PromptCommand, PromptInput,
};

fn main() {
    let wordlist_path = "wordlist.txt";

    let mut solver = match WordleSolver::new(wordlist_path) {
        Ok(solver) => solver,
        Err(e) => {
            eprintln!("Failed to load bundled word list '{wordlist_path}': {e}");
            std::process::exit(1);
        }
    };

    let mut history: Vec<WordleSolver> = Vec::new();
    let mut board_history: Vec<(solver::Word, String)> = Vec::new();

    'game: loop {
        clear_screen();
        println!(
            "========================\n\
             Drew's Wordle Solver\n\
             Enter feedback as 5 chars: G/Y/B (example: GYBBY)\n\
             Commands: HELP STATUS TOP [n] CANDS [n] BOARD UNDO EXIT\n\
             ========================\n"
        );

        render_board(&board_history);
        let remaining = solver.candidates().len();
        println!(
            "Turn {}/{} | Remaining candidates: {}",
            solver.attempt_number(),
            solver.max_attempts(),
            remaining
        );

        let scored = solver.scored_guesses();
        if scored.is_empty() {
            println!("No candidates remain. Check for inconsistent feedback.");
            break;
        }

        let ties = tie_set(&scored, 1e-10);
        let guess = if ties.len() > 1 {
            println!("Multiple top-ranked guesses ({} total):", ties.len());
            for (i, (w, _)) in ties.iter().take(10).enumerate() {
                println!("   {}. {}", i + 1, w);
            }
            if ties.len() > 10 {
                println!("   ... +{} more", ties.len() - 10);
            }
            println!("Pick a guess (number or word), or press Enter for #1.");
            loop {
                let choice = read_line_trimmed();
                if let Some(picked) = parse_choice(&choice, ties) {
                    break picked;
                }
                println!("Invalid choice. Enter a valid number or one of the tied words:");
            }
        } else {
            println!("Suggested guess: {}", scored[0].0);
            scored[0].0
        };

        println!("Enter results for '{guess}': ");
        let results = loop {
            let mut results = String::new();
            io::stdin()
                .read_line(&mut results)
                .expect("Failed to read line");
            match parse_prompt_input(&results) {
                Some(PromptInput::Results(pattern)) => break pattern,
                Some(PromptInput::Command(cmd)) => match cmd {
                    PromptCommand::Exit => return,
                    PromptCommand::Undo => {
                        if let Some(previous) = history.pop() {
                            solver = previous;
                            board_history.pop();
                            println!("Previous turn undone.");
                        } else {
                            println!("Nothing to undo yet.");
                        }
                        println!();
                        continue 'game;
                    }
                    PromptCommand::Help => {
                        println!("Commands:");
                        println!("  HELP         Show this command list");
                        println!("  STATUS       Show turn and candidate status");
                        println!("  TOP [n]      Show top n suggestions (default 10)");
                        println!("  CANDS [n]    Show first n remaining candidates (default 10)");
                        println!("  BOARD        Show guess history with colored feedback");
                        println!("  UNDO         Revert the previous accepted turn");
                        println!("  EXIT         Quit the solver");
                        println!("You can also enter a 5-letter G/Y/B pattern directly.");
                    }
                    PromptCommand::Status => {
                        println!(
                            "Status: turn {}/{} | candidates {} | current guess {}",
                            solver.attempt_number(),
                            solver.max_attempts(),
                            solver.candidates().len(),
                            guess
                        );
                    }
                    PromptCommand::Top(n) => {
                        let limit = n.min(scored.len());
                        if limit == 0 {
                            println!("No scored guesses available.");
                        } else {
                            println!("Top {limit} ranked guesses:");
                            for (i, (w, _)) in scored.iter().take(limit).enumerate() {
                                println!("  {:>2}. {}", i + 1, w);
                            }
                        }
                    }
                    PromptCommand::Cands(n) => {
                        let cands = solver.candidates();
                        let limit = n.min(cands.len());
                        if limit == 0 {
                            println!("No candidates remain.");
                        } else {
                            println!("First {limit} candidates:");
                            for (i, w) in cands.iter().take(limit).enumerate() {
                                println!("  {:>2}. {}", i + 1, w);
                            }
                        }
                    }
                    PromptCommand::Board => {
                        render_board(&board_history);
                    }
                },
                None => {
                    println!("Invalid input. Enter G/Y/B, or type HELP.");
                }
            }
        };

        history.push(solver.clone());
        let status = match solver.next_turn(guess, &results) {
            Ok(status) => status,
            Err(e) => {
                history.pop();
                println!("{e}");
                println!("State unchanged. Please re-enter feedback for the same guess.");
                continue;
            }
        };
        board_history.push((guess, results));

        match status {
            "won" => {
                println!("Congratulations, you won!");
                break;
            }
            "lost" => {
                println!("Game over. Better luck next time!");
                break;
            }
            _ => {}
        }

        println!();
    }
}
