use core::panic;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Stylize},
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TileState {
    Correct,
    Present,
    Absent,
    Unused,
}

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub letter: char,
    pub state: TileState,
}

impl Tile {
    pub fn get_color(&self) -> Color {
        match self.state {
            TileState::Correct => Color::Green,
            TileState::Present => Color::LightYellow,
            TileState::Absent => Color::DarkGray,
            TileState::Unused => Color::Rgb(65, 65, 65),
        }
    }
}

impl Widget for Tile {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::new().bg(self.get_color()).render(area, buf);
        Paragraph::new(format!("{}", self.letter)).bold().render(
            area.centered(Constraint::Length(1), Constraint::Length(1)),
            buf,
        );
    }
}

#[derive(Debug, Clone, Default)]
pub struct Word {
    pub letters: Vec<Tile>,
}

impl Word {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(word: &str) -> Self {
        if word.len() != 5 {
            panic!("word should have length 5");
        }

        let mut ret = Word::new();
        for letter in word.chars() {
            ret.letters.push(Tile {
                letter,
                state: TileState::Absent,
            });
        }
        ret
    }

    fn is_solved(&self) -> bool {
        if self.letters.is_empty() {
            return false;
        }

        for tile in &self.letters {
            if tile.state != TileState::Correct {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_solved_test() {
        let mut w = Word::from("ARISE");
        assert!(!w.is_solved());

        for tile in &mut w.letters {
            tile.state = TileState::Correct;
        }
        w.letters[3].state = TileState::Present;
        assert!(!w.is_solved());

        for tile in &mut w.letters {
            tile.state = TileState::Correct;
        }
        assert!(w.is_solved());
    }
}
