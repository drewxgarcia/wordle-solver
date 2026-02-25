use std::error::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Word(pub(crate) [u8; 5]);

impl Word {
    #[inline]
    pub(crate) fn as_bytes(&self) -> &[u8; 5] {
        &self.0
    }
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Safe because words are normalized to ASCII lowercase at load time.
        let s = std::str::from_utf8(&self.0).unwrap();
        write!(f, "{s}")
    }
}

pub type PatternCode = u16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameStatus {
    Won,
    Lost,
    Ongoing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SolverError {
    InvalidResultsPattern,
    GuessNotInWordList(Word),
    InconsistentFeedback,
}

impl std::fmt::Display for SolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolverError::InvalidResultsPattern => {
                write!(
                    f,
                    "Invalid results: use exactly 5 characters from G, Y, and B"
                )
            }
            SolverError::GuessNotInWordList(word) => {
                write!(f, "Guess '{word}' is not in the solver word list")
            }
            SolverError::InconsistentFeedback => {
                write!(
                    f,
                    "Inconsistent feedback: no candidate matches that guess/pattern pair"
                )
            }
        }
    }
}

impl Error for SolverError {}
