use anyhow::Result;
use core::panic;
use rand::seq::IteratorRandom;
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph, Widget},
};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};

use super::word::{Tile, TileState, Word};

const ROUND: u8 = 6;
const WORD_LEN: usize = 5;
const FILE_PATH: &str = "./words.txt";

enum InputState {
    Guessing,
    Submit,
    Cancel,
    GameEnd,
}

pub struct Wordle {
    round: u8, // maximum 6 rounds
    valid_words: HashSet<String>,
    used_chars: HashMap<char, TileState>,
    answer: String,
    history: Vec<Word>,
    current: String,
    solved: bool,
    err_msg: String,
}

impl Wordle {
    pub fn new() -> Self {
        let valid_words = Wordle::load_words().expect("failed to load words");
        let answer = Wordle::draw_word(&valid_words).expect("failed to draw word");

        let mut used_chars = HashMap::new();
        for i in 'A'..='Z' {
            used_chars.entry(i).or_insert(TileState::Unused);
        }

        Wordle {
            round: 1,
            valid_words,
            used_chars,
            answer,
            history: Vec::new(),
            current: String::new(),
            solved: false,
            err_msg: String::new(),
        }
    }

    fn load_words() -> Result<HashSet<String>> {
        let file = File::open(FILE_PATH)?;
        let reader: BufReader<File> = BufReader::new(file);

        let mut string_set = HashSet::new();

        for line in reader.lines() {
            string_set.insert(line?.to_ascii_uppercase());
        }

        Ok(string_set)
    }

    fn draw_word(words: &HashSet<String>) -> Option<String> {
        if words.is_empty() {
            panic!("Error: empty word set");
        }

        let mut rng = rand::rng();
        words.iter().choose(&mut rng).cloned()
    }

    fn game_restart(&mut self) {
        self.round = 1;
        for (_, state) in self.used_chars.iter_mut() {
            *state = TileState::Unused;
        }
        self.answer = Wordle::draw_word(&self.valid_words).expect("failed to draw word");
        self.history = Vec::new();
        self.current = String::new();
        self.solved = false;
        self.err_msg = String::new();
    }

    fn handle_input(&mut self) -> InputState {
        if let Ok(Event::Key(key)) = event::read() {
            match key.code {
                KeyCode::Esc => return InputState::Cancel,
                KeyCode::Tab => {
                    self.game_restart();
                }
                KeyCode::Char(ch) if self.round <= 6 && !self.solved => {
                    if self.current.len() < 5 {
                        self.current.push(ch.to_ascii_uppercase());
                    }
                }
                KeyCode::Backspace if self.round <= 6 && !self.solved => {
                    if !self.current.is_empty() {
                        self.current.pop();
                    }
                }
                KeyCode::Enter if self.round <= 6 && !self.solved => {
                    return InputState::Submit;
                }
                _ => return InputState::GameEnd, // if round >= 6, ignore all input except esc
            }
        }
        InputState::Guessing
    }

    fn parse_input(&self, input: &str) -> Result<Word, String> {
        let trimmed_input = input.trim();

        if !trimmed_input.is_ascii() {
            return Err(String::from("not ascii"));
        }

        if trimmed_input.len() != WORD_LEN {
            return Err(String::from("incorrect word length"));
        }

        if !self
            .valid_words
            .contains(&trimmed_input.to_ascii_uppercase())
        {
            return Err(String::from("invalid word"));
        }

        Ok(Word::from(&trimmed_input.to_ascii_uppercase()))
    }

    fn compare(&self, user_input: &mut Word) {
        let mut answer_map: HashMap<char, u8> = HashMap::new();
        self.answer.chars().for_each(|c| {
            *answer_map.entry(c).or_insert(0) += 1;
        });

        // check correct letters
        let answer_vec: Vec<char> = self.answer.chars().collect();
        for (i, tile) in user_input.letters.iter_mut().enumerate() {
            if answer_vec[i] == tile.letter {
                tile.state = TileState::Correct;
                if let Some(val) = answer_map.get_mut(&tile.letter) {
                    *val -= 1;
                }
            }
        }

        // check present and absent letters
        for tile in user_input.letters.iter_mut() {
            if tile.state == TileState::Correct {
                continue;
            }

            match answer_map.get_mut(&tile.letter) {
                Some(val) if *val > 0 => {
                    tile.state = TileState::Present;
                    *val -= 1;
                }
                _ => tile.state = TileState::Absent,
            }
        }
    }

    fn update_status(&mut self, result: &Word) {
        self.history.push(result.clone());
        let mut solved: bool = true;

        // update used chars
        for tile in result.letters.iter() {
            if tile.state != TileState::Correct {
                solved = false;
            }

            let used_state = self
                .used_chars
                .entry(tile.letter)
                .or_insert(TileState::Unused);
            if *used_state == TileState::Unused
                || (*used_state == TileState::Present && tile.state == TileState::Correct)
            {
                *used_state = tile.state;
            }
        }

        // update status
        self.solved = solved;
    }

    fn update_screen(&self, frame: &mut Frame) {
        let [outer] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(50)])
            .margin(1)
            .areas(
                frame
                    .area()
                    .centered(Constraint::Length(50), Constraint::Length(41)),
            );

        let [msg, top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(2),
                Constraint::Length(25),
                Constraint::Length(8),
            ])
            .margin(2)
            .areas(outer);

        /* border */
        let instructions = Line::from(vec![
            " Submit ".into(),
            "<Enter>".blue().bold(),
            " New game ".into(),
            "<Tab>".blue().bold(),
            " Quit ".into(),
            "<Esc>".blue().bold(),
        ]);

        Block::bordered()
            .title("Wordle")
            .title_bottom(instructions.right_aligned())
            .border_type(BorderType::Rounded)
            .render(outer, frame.buffer_mut());

        /* game board */
        Block::bordered()
            .border_type(BorderType::Rounded)
            .render(top, frame.buffer_mut());
        let [game_board_area] = Layout::vertical([Constraint::Fill(1)]).margin(1).areas(top);

        if !self.err_msg.is_empty() {
            let span = Span::styled(self.err_msg.clone(), Style::default().fg(Color::Red));
            frame.render_widget(span, msg);
        }

        // past guesses
        let width: u16 = 5;
        let height: u16 = 3;
        let center_x = (game_board_area.left() + game_board_area.right()) / 2;
        for (row, word) in self.history.iter().enumerate() {
            for (col, tile) in word.letters.iter().enumerate() {
                let area = Rect {
                    x: (center_x as i32 - width as i32 / 2
                        + ((col as i32 - 2) * (width + 2) as i32)) as u16,
                    y: game_board_area.y + (row as i32 * (height + 1) as i32) as u16,
                    width,
                    height,
                };
                tile.render(area, frame.buffer_mut());
            }
        }

        // current guess
        for (col, ch) in self.current.chars().enumerate() {
            let area = Rect {
                x: (center_x as i32 - width as i32 / 2 + ((col as i32 - 2) * (width + 2) as i32))
                    as u16,
                y: game_board_area.y + (self.history.len() as i32 * (height + 1) as i32) as u16,
                width,
                height,
            };
            let tile = Tile {
                letter: ch,
                state: TileState::Absent,
            };
            tile.render(area, frame.buffer_mut());
        }

        /* keyboard */
        let qwerty = [
            "Q W E R T Y U I O P",
            " A S D F G H J K L ",
            "  Z X C V B N M    ",
        ];
        Block::bordered()
            .border_type(BorderType::Rounded)
            .render(bottom, frame.buffer_mut());
        let [keyboard_area] = Layout::vertical([Constraint::Fill(1)])
            .margin(1)
            .areas(bottom);
        let mut lines = Vec::new();
        for row in qwerty {
            let spans = row
                .chars()
                .map(|ch| {
                    if ch == ' ' {
                        Span::raw(" ")
                    } else {
                        let color = match self.used_chars.get(&ch) {
                            Some(TileState::Correct) => Color::Green,
                            Some(TileState::Present) => Color::Yellow,
                            Some(TileState::Absent) => Color::DarkGray,
                            Some(TileState::Unused) => Color::Black,
                            _ => unreachable!(),
                        };
                        Span::styled(
                            format!(" {ch} "),
                            Style::default().bg(color).add_modifier(Modifier::BOLD),
                        )
                    }
                })
                .collect();
            lines.push(spans);
            lines.push(Line::from(vec![Span::default()]));
        }
        let keyboard = Paragraph::new(lines).alignment(Alignment::Center);
        frame.render_widget(keyboard, keyboard_area);

        if self.solved || self.round > ROUND {
            let game_result = if self.solved {
                vec![
                    Span::styled(
                        "You won! The answer is: ",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Green),
                    ),
                    Span::styled(
                        &self.answer,
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::White),
                    ),
                ]
            } else {
                vec![
                    Span::styled(
                        "You lost! The answer is: ",
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::LightYellow),
                    ),
                    Span::styled(
                        &self.answer,
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::White),
                    ),
                ]
            };

            let popup_area = frame
                .area()
                .centered(Constraint::Length(40), Constraint::Length(4));
            frame.render_widget(Clear, popup_area);

            let new_game = vec![
                Span::raw("New Game? "),
                Span::styled("<Tab>", Style::default().blue().bold()),
            ];
            let popup = Paragraph::new(vec![game_result.into(), new_game.into()])
                .block(Block::bordered())
                .alignment(Alignment::Center);

            frame.render_widget(popup, popup_area);
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();

        loop {
            terminal.draw(|frame| {
                self.update_screen(frame);
            })?;

            match self.handle_input() {
                InputState::Submit => {
                    // parsing
                    let mut guess = match self.parse_input(&self.current) {
                        Ok(val) => {
                            self.err_msg.clear();
                            val
                        }
                        Err(err) => {
                            self.err_msg = err;
                            continue;
                        }
                    };

                    // compare
                    self.compare(&mut guess);

                    // update game status
                    self.update_status(&guess);

                    self.round += 1;

                    self.current.clear();
                }
                InputState::Cancel => break,
                InputState::Guessing | InputState::GameEnd => {}
            }
        }
        ratatui::restore();

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn draw_word_empty_test() {
        let words = HashSet::new();
        Wordle::draw_word(&words);
    }

    #[test]
    fn draw_word_test() {
        let words = HashSet::from([
            "this".to_string(),
            "is".to_string(),
            "a".to_string(),
            "test".to_string(),
        ]);
        let result = Wordle::draw_word(&words).expect("random word expected");
        assert!(words.contains(&result));
    }

    #[test]
    fn load_words_test() {
        let words = Wordle::load_words().expect("words expected");
        assert!(!words.is_empty());

        for word in &words {
            assert!(!word.is_empty());
            assert!(word.chars().all(|ch| ch.is_ascii_uppercase()));
        }
    }

    #[test]
    fn compare_test() {
        let mut game = Wordle::new();
        game.answer = "CRATE".to_string();
        let mut guess = Word::from("CATER");
        game.compare(&mut guess);
        assert_eq!(
            guess
                .letters
                .into_iter()
                .map(|tile| tile.state)
                .collect::<Vec<TileState>>(),
            vec![
                TileState::Correct,
                TileState::Present,
                TileState::Present,
                TileState::Present,
                TileState::Present,
            ]
        );

        let mut game = Wordle::new();
        game.answer = "HOUND".to_string();
        let mut guess = Word::from("AMONG");
        game.compare(&mut guess);
        assert_eq!(
            guess
                .letters
                .into_iter()
                .map(|tile| tile.state)
                .collect::<Vec<TileState>>(),
            [
                TileState::Absent,
                TileState::Absent,
                TileState::Present,
                TileState::Correct,
                TileState::Absent,
            ]
        );

        let mut game = Wordle::new();
        game.answer = "TRAIT".to_string();
        let mut guess = Word::from("TXTXT");
        game.compare(&mut guess);
        assert_eq!(
            guess
                .letters
                .into_iter()
                .map(|tile| tile.state)
                .collect::<Vec<TileState>>(),
            [
                TileState::Correct,
                TileState::Absent,
                TileState::Absent,
                TileState::Absent,
                TileState::Correct,
            ]
        );

        let mut game = Wordle::new();
        game.answer = "TRAIT".to_string();
        let mut guess = Word::from("TXTTX");
        game.compare(&mut guess);
        assert_eq!(
            guess
                .letters
                .into_iter()
                .map(|tile| tile.state)
                .collect::<Vec<TileState>>(),
            [
                TileState::Correct,
                TileState::Absent,
                TileState::Present,
                TileState::Absent,
                TileState::Absent,
            ]
        );
    }

    #[test]
    fn update_status_test() {
        let mut game = Wordle::new();
        game.answer = "DEALT".to_string();
        let mut guess = Word::from("ASIDE");
        game.compare(&mut guess);
        game.update_status(&guess);
        for (&ch, &state) in &game.used_chars {
            if ch == 'A' || ch == 'D' || ch == 'E' {
                assert!(state == TileState::Present);
            } else if ch == 'S' || ch == 'I' {
                assert!(state == TileState::Absent);
            } else {
                assert!(state == TileState::Unused);
            }
        }
        assert!(!game.solved);

        let mut guess = Word::from("DEATH");
        game.compare(&mut guess);
        game.update_status(&guess);
        for (&ch, &state) in &game.used_chars {
            if ch == 'D' || ch == 'E' || ch == 'A' {
                assert!(state == TileState::Correct);
            } else if ch == 'T' {
                assert!(state == TileState::Present);
            } else if ch == 'H' || ch == 'S' || ch == 'I' {
                assert!(state == TileState::Absent);
            } else {
                assert!(state == TileState::Unused);
            }
        }
        assert!(!game.solved);

        let mut guess = Word::from("DEALT");
        game.compare(&mut guess);
        game.update_status(&guess);
        for (&ch, &state) in &game.used_chars {
            if ch == 'D' || ch == 'E' || ch == 'A' || ch == 'L' || ch == 'T' {
                assert!(state == TileState::Correct);
            } else if ch == 'H' || ch == 'S' || ch == 'I' {
                assert!(state == TileState::Absent);
            } else {
                assert!(state == TileState::Unused);
            }
        }
        assert!(game.solved);
    }
}
