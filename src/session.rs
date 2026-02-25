use crate::solver::{GameStatus, PatternCode, SolverError, Word, WordleSolver};
use std::io;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Turn {
    pub guess: Word,
    pub feedback: PatternCode,
}

pub struct GameSession {
    initial_solver: WordleSolver,
    solver: WordleSolver,
    turns: Vec<Turn>,
}

impl GameSession {
    pub fn new(wordlist_path: &str) -> io::Result<Self> {
        let solver = WordleSolver::new(wordlist_path)?;
        Ok(Self {
            initial_solver: solver.clone(),
            solver,
            turns: Vec::new(),
        })
    }

    pub fn solver(&self) -> &WordleSolver {
        &self.solver
    }

    pub fn turns(&self) -> &[Turn] {
        &self.turns
    }

    pub fn apply_turn(
        &mut self,
        guess: Word,
        feedback: PatternCode,
    ) -> Result<GameStatus, SolverError> {
        let status = self.solver.next_turn(guess, feedback)?;
        self.turns.push(Turn { guess, feedback });
        Ok(status)
    }

    fn replay_turns(&self) -> Result<WordleSolver, SolverError> {
        let mut rebuilt = self.initial_solver.clone();
        for turn in &self.turns {
            rebuilt.next_turn(turn.guess, turn.feedback)?;
        }
        Ok(rebuilt)
    }

    pub fn undo(&mut self) -> bool {
        let popped_turn = match self.turns.pop() {
            Some(turn) => turn,
            None => return false,
        };

        if let Ok(rebuilt) = self.replay_turns() {
            self.solver = rebuilt;
            true
        } else {
            self.turns.push(popped_turn);
            false
        }
    }
}
