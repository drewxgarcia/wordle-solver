use super::oracle::parse_word5;
use super::types::Word;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub fn load_word_list(word_list_path: &str) -> io::Result<Vec<Word>> {
    let file = File::open(word_list_path)?;
    let reader = BufReader::new(file);

    let mut seen = HashSet::new();
    let mut words = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let raw = line?;
        let line_no = idx + 1;
        let Some(word) = parse_word5(&raw) else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid word at line {line_no}: expected exactly 5 ASCII letters"),
            ));
        };

        // Preserve original order while dropping duplicates.
        if seen.insert(word) {
            words.push(word);
        }
    }

    if words.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Word list is empty after validation",
        ));
    }

    Ok(words)
}
