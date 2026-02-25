mod modes;
mod session;
mod solver;
mod ui;

use crate::modes::{common, game, solver as solver_mode};
use crate::ui::{clear_screen, read_line_trimmed};

const WORDLIST_PATH: &str = "wordlist.txt";

enum MainMenuChoice {
    Game,
    Solver,
    Help,
    Exit,
}

fn parse_main_menu_choice(input: &str) -> Option<MainMenuChoice> {
    match input.trim().to_ascii_uppercase().as_str() {
        "1" | "PLAY" | "GAME" => Some(MainMenuChoice::Game),
        "2" | "SOLVER" | "SOLVE" => Some(MainMenuChoice::Solver),
        "3" | "HELP" => Some(MainMenuChoice::Help),
        "4" | "EXIT" | "QUIT" => Some(MainMenuChoice::Exit),
        _ => None,
    }
}

fn show_help_screen() {
    clear_screen();
    println!(
        "========================\n\
         Help\n\
         ========================\n"
    );
    println!("1) Game Mode");
    println!("   You guess words; the game computes G/Y/B feedback.");
    println!("   Use commands like /HINT and /UNDO while playing.");
    println!();
    println!("2) Solver Mode");
    println!("   The solver suggests guesses; you type Wordle feedback (G/Y/B).");
    println!("   Great when you're solving an external game.");
    println!();
    println!("All words come from the bundled word list: {WORDLIST_PATH}");
    common::wait_for_enter("Press Enter to return to the main menu.");
}

fn main() {
    loop {
        clear_screen();
        println!(
            "========================\n\
             Drew's Wordle Arcade\n\
             ========================\n"
        );
        println!("1) Play Wordle (Game Mode)");
        println!("2) Solve an External Wordle (Solver Mode)");
        println!("3) Help");
        println!("4) Exit");
        println!();
        println!("Choose an option:");

        let choice_input = match read_line_trimmed() {
            Ok(Some(input)) => input,
            Ok(None) => {
                println!("EOF received. Exiting.");
                break;
            }
            Err(e) => {
                eprintln!("Failed to read input: {e}");
                break;
            }
        };

        let Some(choice) = parse_main_menu_choice(&choice_input) else {
            println!("Invalid selection.");
            common::wait_for_enter("Press Enter to continue.");
            continue;
        };

        let outcome = match choice {
            MainMenuChoice::Game => game::run(WORDLIST_PATH),
            MainMenuChoice::Solver => solver_mode::run(WORDLIST_PATH),
            MainMenuChoice::Help => {
                show_help_screen();
                Ok(())
            }
            MainMenuChoice::Exit => break,
        };

        if let Err(e) = outcome {
            eprintln!("Error: {e}");
            common::wait_for_enter("Press Enter to return to the main menu.");
        }
    }
}
