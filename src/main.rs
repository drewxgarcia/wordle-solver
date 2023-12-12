use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::sync::mpsc;
use std::thread;
use std::sync::Arc;

fn simulate_results(guess: &str, target: &str) -> String {
    let mut results = vec!['B'; guess.len()];
    let mut target_count: HashMap<char, i32> = HashMap::new();
    for c in target.chars() {
        *target_count.entry(c).or_insert(0) += 1;
    }

    for (i, (g, t)) in guess.chars().zip(target.chars()).enumerate() {
        if g == t {
            results[i] = 'G';
            *target_count.get_mut(&g).unwrap() -= 1;
        } else if target_count.get(&g).unwrap_or(&0) > &0 {
            results[i] = 'Y';
            *target_count.get_mut(&g).unwrap() -= 1;
        }
    }

    let mut result_string = String::with_capacity(guess.len());
    result_string.extend(results.iter());
    result_string
}

fn calculate_entropy(word: &str, word_list: &[String]) -> f64 {
    let total_words = word_list.len() as f64;

    // Create pattern counts using an iterator and collect them into a hashmap
    let pattern_counts = word_list
        .iter()
        .map(|target| simulate_results(word, target))
        .fold(HashMap::new(), |mut acc, pattern| {
            *acc.entry(pattern).or_insert(0) += 1;
            acc
        });

    // Calculate entropy using an iterator
    -pattern_counts.values().fold(0.0, |acc, &count| {
        let probability = count as f64 / total_words;
        acc + probability * probability.log2()
    })
}

struct WordleSolver {
    word_list: Vec<String>,
    known_correct: HashMap<usize, char>,
    known_wrong_positions: HashMap<char, HashSet<usize>>,
    known_absent: HashSet<char>,
    attempts: usize,
    max_attempts: usize,
    current_guess: Option<String>
}

impl WordleSolver {
    fn new(word_list_path: &str) -> io::Result<Self> {
        let file = File::open(word_list_path)?;
        let reader = BufReader::new(file);
        let word_list: Vec<String> = reader
            .lines()
            .filter_map(|line| {
                let word = line.ok()?;
                if word.len() == 5 {
                    Some(word.to_lowercase())
                } else {
                    None
                }
            })
            .collect();

        Ok(Self {
            word_list,
            known_correct: HashMap::new(),
            known_wrong_positions: HashMap::new(),
            known_absent: HashSet::new(),
            attempts: 0,
            max_attempts: 6,
            current_guess: None
        })
    }

    // In make_guess method, wrap word_list in an Arc for shared ownership
    fn make_guess(&self) -> Option<String> {
        let (tx, rx) = mpsc::channel();
        let word_list = Arc::new(self.word_list.clone()); // Clone the word list itself into the Arc
    
        thread::spawn(move || {
            for word in word_list.iter() { // Iterate over the word list
                let tx = tx.clone();
                let word_clone = word.clone(); // Clone the word
                let word_list_clone = Arc::clone(&word_list); // Clone the Arc, not the list
                thread::spawn(move || {
                    let entropy = calculate_entropy(&word_clone, &word_list_clone);
                    tx.send((word_clone, entropy)).unwrap();
                });
            }
        });
    
        let mut max_entropy = (None, f64::MIN);
        for (word, entropy) in rx {
            if entropy > max_entropy.1 {
                max_entropy = (Some(word), entropy);
            }
        }
    
        max_entropy.0
    }    

    fn process_results(&mut self, guess: &str, results: &str) {
        guess.chars().zip(results.chars()).enumerate().for_each(|(idx, (letter, status))| {
            match status {
                'G' => { self.known_correct.insert(idx, letter); }
                'Y' => { self.known_wrong_positions.entry(letter).or_insert_with(HashSet::new).insert(idx); }
                'B' => { 
                    if !self.known_correct.values().any(|&v| v == letter) && !self.known_wrong_positions.contains_key(&letter) {
                        self.known_absent.insert(letter);
                    }
                }
                _ => {}
            }
        });

        self.word_list = self.word_list.iter().filter(|&word| self.is_possible_word(word)).cloned().collect();
    }

    fn is_possible_word(&self, word: &str) -> bool {
        self.known_correct.iter().all(|(&idx, &letter)| word.chars().nth(idx).map_or(false, |w| w == letter))
            && self.known_wrong_positions.iter().all(|(&letter, positions)| 
                word.contains(letter) && positions.iter().all(|&idx| word.chars().nth(idx).unwrap_or('\0') != letter))
            && self.known_absent.iter().all(|&letter| !word.contains(letter))
    }

    fn check_game_status(&self, results: &str) -> String {
        if results.chars().all(|c| c == 'G') {
            "won".to_string()
        } else if self.attempts >= self.max_attempts {
            "lost".to_string()
        } else {
            "ongoing".to_string()
        }
    }

    fn next_turn(&mut self, results: &str) -> String {
        // Temporarily take the current_guess out of self to avoid mutable-immutable borrow conflict
        let current_guess = self.current_guess.take();
        if let Some(ref guess) = current_guess {
            self.process_results(guess, results);
        }
        self.attempts += 1;
        let game_status = self.check_game_status(results);

        if game_status == "ongoing" {
            self.current_guess = self.make_guess();
        } else {
            self.current_guess = None;
        }

        // Put the current_guess back in case it was taken out
        if self.current_guess.is_none() {
            self.current_guess = current_guess;
        }

        game_status
    }
}

// Main function
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: wordle_solver <wordlist_path>");
        std::process::exit(1);
    }
    let wordlist_path = &args[1];

    let mut solver = match WordleSolver::new(wordlist_path) {
        Ok(solver) => solver,
        Err(e) => {
            eprintln!("Failed to load word list: {}", e);
            std::process::exit(1);
        }
    };

    // Startup instructions
    println!(
        "·▄▄▄▄  ▄▄▄  ▄▄▄ .▄▄▌ ▐ ▄▌.▄▄ ·      \n\
         ██▪ ██ ▀▄ █·▀▄.▀·██· █▌▐█▐█ ▀.      \n\
         ▐█· ▐█▌▐▀▀▄ ▐▀▀▪▄██▪▐█▐▐▌▄▀▀▀█▄     \n\
         ██. ██ ▐█•█▌▐█▄▄▌▐█▌██▐█▌▐█▄▪▐█     \n\
         ▀▀▀▀▀• .▀  ▀ ▀▀▀  ▀▀▀▀ ▀▪ ▀▀▀▀      \n\
         ▄▄▌ ▐ ▄▌      ▄▄▄  ·▄▄▄▄  ▄▄▌  ▄▄▄ .\n\
         ██· █▌▐█▪     ▀▄ █·██▪ ██ ██•  ▀▄.▀·\n\
         ██▪▐█▐▐▌ ▄█▀▄ ▐▀▀▄ ▐█· ▐█▌██▪  ▐▀▀▪▄\n\
         ▐█▌██▐█▌▐█▌.▐▌▐█•█▌██. ██ ▐█▌▐▌▐█▄▄▌\n\
          ▀▀▀▀ ▀▪ ▀█▄▀▪.▀  ▀▀▀▀▀▀• .▀▀▀  ▀▀▀ \n\
         .▄▄ ·       ▄▄▌   ▌ ▐·▄▄▄ .▄▄▄      \n\
         ▐█ ▀. ▪     ██•  ▪█·█▌▀▄.▀·▀▄ █·    \n\
         ▄▀▀▀█▄ ▄█▀▄ ██▪  ▐█▐█•▐▀▀▪▄▐▀▀▄     \n\
         ▐█▄▪▐█▐█▌.▐▌▐█▌▐▌ ███ ▐█▄▄▌▐█•█▌    \n\
          ▀▀▀▀  ▀█▄▀▪.▀▀▀ . ▀   ▀▀▀ .▀  ▀    \n\
         ========================\n\
         Welcome to Drew's Wordle solver! Here's how to use it:\n\
         * Type in the results of each guess as a string of 'G', 'Y', and 'B'.\n\
         * 'G' for Green (correct position)\n\
         * 'Y' for Yellow (wrong position)\n\
         * 'B' for Black (not in the word)\n\
         * Press enter to submit the results to the solver\n\
         * Type 'EXIT' to quit the game\n\
         ========================\n"
    );

    solver.current_guess = solver.make_guess();
    println!("The solver's initial guess is: {}", solver.current_guess.as_ref().unwrap());

    loop {
        let mut results = String::new();
        println!("Enter results for '{}': ", solver.current_guess.as_ref().unwrap());
        io::stdin().read_line(&mut results).expect("Failed to read line");
        let results = results.trim().to_uppercase();

        if results == "EXIT" {
            break;
        }
        if !valid_results(&results) {
            println!("Invalid results. Please enter a 5-letter string of 'G', 'Y', and 'B'.");
            continue;
        }

        let game_status = solver.next_turn(&results);
        if game_status == "won" {
            println!("Congratulations, you won!");
            break;
        } else if game_status == "lost" {
            println!("Game over. Better luck next time!");
            break;
        } else {
            println!("Next guess: {}", solver.current_guess.as_ref().unwrap());
        }
    }
}

// Additional helper functions
fn valid_results(results: &str) -> bool {
    results.len() == 5 && results.chars().all(|c| matches!(c, 'G' | 'Y' | 'B'))
}