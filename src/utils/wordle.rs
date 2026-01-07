use anyhow::Result;
use core::panic;
use rand::seq::IteratorRandom;
use regex::Regex;
use reqwest::blocking;
use std::collections::{HashMap, HashSet};

use super::ui::InputState;
use super::word::{TileState, Word};

pub const ROUND: u8 = 6;
const WORD_LEN: usize = 5;

pub struct Wordle {
    pub round: u8, // maximum 6 rounds
    pub valid_words: HashSet<String>,
    pub used_chars: HashMap<char, TileState>,
    pub answer: String,
    pub history: Vec<Word>,
    pub current: String,
    pub solved: bool,
    pub err_msg: String,
    pub show_def: bool,
    pub end_game: bool,
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
            end_game: false,
            show_def: false,
        }
    }

    fn load_words() -> Result<HashSet<String>> {
        const WORDS: &str = include_str!("../../words.txt");
        let words: Vec<&str> = WORDS.lines().collect();

        let mut result = HashSet::new();
        words.iter().for_each(|w| {
            result.insert(w.to_ascii_uppercase());
        });
        Ok(result)
    }

    fn draw_word(words: &HashSet<String>) -> Option<String> {
        if words.is_empty() {
            panic!("Error: empty word set");
        }

        let mut rng = rand::rng();
        words.iter().choose(&mut rng).cloned()
    }

    pub fn game_restart(&mut self) {
        self.round = 1;
        for (_, state) in self.used_chars.iter_mut() {
            *state = TileState::Unused;
        }
        self.answer = Wordle::draw_word(&self.valid_words).expect("failed to draw word");
        self.history = Vec::new();
        self.current.clear();
        self.solved = false;
        self.err_msg.clear();
        self.end_game = false;
        self.show_def = false;
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

        // First pass: mark correct letters
        for (i, tile) in user_input.letters.iter_mut().enumerate() {
            if answer_vec[i] == tile.letter {
                tile.state = TileState::Correct;
                if let Some(val) = answer_map.get_mut(&tile.letter) {
                    *val -= 1;
                }
            }
        }

        // Second pass: mark present and absent letters
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
        let mut solved = true;

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

    pub fn get_word_def(&self, word: &String) -> Option<Vec<String>> {
        let url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", word);
        let content = blocking::get(url).ok()?.text().ok()?;

        let re = Regex::new(r#""definition":"([^"]*)""#).ok()?;
        let mut definition = vec![format!("\nDefinitions for '{}':\n", word)];
        for cap in re.captures_iter(&content) {
            definition.push("- ".to_string() + &cap[1]);
        }
        Some(definition)
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
                InputState::Guessing | InputState::None => {}
            }

            if self.round > 6 || self.solved {
                self.end_game = true;
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
