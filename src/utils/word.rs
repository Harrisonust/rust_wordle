use core::panic;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Stylize},
    widgets::{Block, Paragraph, Widget},
};

pub const WORD_LEN: usize = 5;

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
            TileState::Present => Color::Yellow,
            TileState::Absent => Color::DarkGray,
            TileState::Unused => Color::Rgb(65, 65, 65), // very dark gray
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
