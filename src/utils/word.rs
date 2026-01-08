use super::tile::{Tile, TileState};
use core::panic;

pub const WORD_LEN: usize = 5;

#[derive(Debug, Clone, Default)]
pub struct Word {
    pub letters: Vec<Tile>,
}

impl Word {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(word: &str) -> Self {
        if word.len() != WORD_LEN {
            panic!("{}", format!("word should have length {}", WORD_LEN));
        }

        let mut ret = Word::new();
        word.chars().for_each(|letter| {
            ret.letters.push(Tile {
                letter,
                state: TileState::Absent,
            });
        });
        ret
    }
}

#[cfg(test)]
mod test {}
