use anyhow::Result;
use core::panic;
use rand::seq::IteratorRandom;
use ratatui::{
    Frame,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph, Widget},
};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};

const ROUND: u8 = 6;
const WORD_LEN: usize = 5;
const FILE_PATH: &str = "./words.txt";

#[derive(Debug, PartialEq, Clone, Copy)]
enum State {
    Correct,
    Present,
    Absent,
    Unused,
}

enum InputState {
    Default,
    Submit,
    Cancel,
}

#[derive(Debug, Clone, Copy)]
struct Tile {
    letter: char,
    state: State,
}

impl Tile {
    fn get_color(&self) -> Color {
        match self.state {
            State::Correct => Color::Green,
            State::Present => Color::LightYellow,
            State::Absent => Color::DarkGray,
            State::Unused => Color::Gray,
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

#[derive(Debug, Clone)]
struct Word {
    letters: Vec<Tile>,
}

impl Word {
    fn new() -> Self {
        Word {
            letters: Vec::new(),
        }
    }

    fn from(word: &str) -> Self {
        if word.len() != 5 {
            panic!("word should have length 5");
        }

        let mut ret = Word::new();
        for letter in word.chars() {
            ret.letters.push(Tile {
                letter,
                state: State::Absent,
            });
        }
        ret
    }

    fn is_solved(&self) -> bool {
        if self.letters.is_empty() {
            return false;
        }

        for tile in &self.letters {
            if tile.state != State::Correct {
                return false;
            }
        }
        true
    }
}

struct Wordle {
    round: u8, // maximum 6 rounds
    valid_words: HashSet<String>,
    used_chars: HashMap<char, State>,
    answer: String,
    history: Vec<Word>,
    current: String,
    solved: bool,
    err_msg: String,
}

impl Wordle {
    fn new() -> Self {
        let valid_words = Wordle::load_words().expect("failed to load words");
        let answer = Wordle::draw_word(&valid_words).expect("failed to draw word");

        let mut used_chars = HashMap::new();
        for i in 'A'..='Z' {
            used_chars.entry(i).or_insert(State::Unused);
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

    fn handle_input(&mut self) -> InputState {
        if let Ok(Event::Key(key)) = event::read() {
            match key.code {
                KeyCode::Esc => return InputState::Cancel,
                KeyCode::Char(ch) => {
                    if self.current.len() < 5 {
                        self.current.push(ch.to_ascii_uppercase());
                    }
                }
                KeyCode::Backspace => {
                    if !self.current.is_empty() {
                        self.current.pop();
                    }
                }
                KeyCode::Enter => {
                    return InputState::Submit;
                }
                _ => {}
            }
        }
        InputState::Default
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
                tile.state = State::Correct;
                if let Some(val) = answer_map.get_mut(&tile.letter) {
                    *val -= 1;
                }
            }
        }

        // check present and absent letters
        for tile in user_input.letters.iter_mut() {
            if tile.state == State::Correct {
                continue;
            }

            match answer_map.get_mut(&tile.letter) {
                Some(val) if *val > 0 => {
                    tile.state = State::Present;
                    *val -= 1;
                }
                _ => tile.state = State::Absent,
            }
        }
    }

    fn update_status(&mut self, result: &Word) {
        self.history.push(result.clone());
        let mut solved: bool = true;

        // update keyboard
        for tile in result.letters.iter() {
            if tile.state != State::Correct {
                solved = false;
            }

            let used_state = self.used_chars.entry(tile.letter).or_insert(State::Unused);
            if *used_state == State::Unused
                || (*used_state == State::Present && tile.state == State::Correct)
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
            .areas(frame.area().centered_horizontally(Constraint::Length(50)));

        let [msg, top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(5),
                Constraint::Percentage(60),
                Constraint::Percentage(35),
            ])
            .margin(1)
            .areas(outer);

        /* border */
        let instructions = Line::from(vec![
            " Submit ".into(),
            "<Enter>".blue().bold(),
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
            let span = Span::styled(self.err_msg.clone(), Style::default().fg(Color::Yellow));
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
                state: State::Absent,
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
                            Some(State::Correct) => Color::Green,
                            Some(State::Present) => Color::Yellow,
                            Some(State::Absent) => Color::DarkGray,
                            Some(State::Unused) => Color::Black,
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
                .centered(Constraint::Length(40), Constraint::Percentage(20));
            frame.render_widget(Clear, popup_area);

            let popup_para = Paragraph::new(vec![game_result.into()])
                .block(Block::bordered())
                .alignment(Alignment::Center);

            frame.render_widget(popup_para, popup_area);
        }
    }

    fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();

        loop {
            terminal.draw(|frame| {
                self.update_screen(frame);
            })?;

            match self.handle_input() {
                InputState::Default => {}
                InputState::Submit => {
                    if self.round > 6 {
                        continue;
                    }

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
            }
        }
        ratatui::restore();

        Ok(())
    }
}

fn main() {
    let mut game = Wordle::new();

    dbg!("log answer here: {}", &game.answer);

    if let Err(e) = game.run() {
        eprintln!("{:#}", e);
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
            tile.state = State::Correct;
        }
        w.letters[3].state = State::Present;
        assert!(!w.is_solved());

        for tile in &mut w.letters {
            tile.state = State::Correct;
        }
        assert!(w.is_solved());
    }

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
                .collect::<Vec<State>>(),
            vec![
                State::Correct,
                State::Present,
                State::Present,
                State::Present,
                State::Present,
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
                .collect::<Vec<State>>(),
            [
                State::Absent,
                State::Absent,
                State::Present,
                State::Correct,
                State::Absent,
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
                .collect::<Vec<State>>(),
            [
                State::Correct,
                State::Absent,
                State::Absent,
                State::Absent,
                State::Correct,
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
                .collect::<Vec<State>>(),
            [
                State::Correct,
                State::Absent,
                State::Present,
                State::Absent,
                State::Absent,
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
                assert!(state == State::Present);
            } else if ch == 'S' || ch == 'I' {
                assert!(state == State::Absent);
            } else {
                assert!(state == State::Unused);
            }
        }
        assert!(!game.solved);

        let mut guess = Word::from("DEATH");
        game.compare(&mut guess);
        game.update_status(&guess);
        for (&ch, &state) in &game.used_chars {
            if ch == 'D' || ch == 'E' || ch == 'A' {
                assert!(state == State::Correct);
            } else if ch == 'T' {
                assert!(state == State::Present);
            } else if ch == 'H' || ch == 'S' || ch == 'I' {
                assert!(state == State::Absent);
            } else {
                assert!(state == State::Unused);
            }
        }
        assert!(!game.solved);

        let mut guess = Word::from("DEALT");
        game.compare(&mut guess);
        game.update_status(&guess);
        for (&ch, &state) in &game.used_chars {
            if ch == 'D' || ch == 'E' || ch == 'A' || ch == 'L' || ch == 'T' {
                assert!(state == State::Correct);
            } else if ch == 'H' || ch == 'S' || ch == 'I' {
                assert!(state == State::Absent);
            } else {
                assert!(state == State::Unused);
            }
        }
        assert!(game.solved);
    }
}
