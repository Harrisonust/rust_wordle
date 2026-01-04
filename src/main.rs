use anyhow::{Result, anyhow};
use colored::Colorize;
use core::panic;
use rand::seq::IteratorRandom;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

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

#[derive(Debug, Clone)]
struct Word {
    letters: Vec<(char, State)>,
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (c, state) in &self.letters {
            match *state {
                State::Correct => {
                    write!(f, "{}", c.to_string().green())?;
                }
                State::Absent => {
                    write!(f, "{}", c.to_string().white())?;
                }
                State::Present => {
                    write!(f, "{}", c.to_string().yellow())?;
                }
                State::Unused => {
                    write!(f, "{}", c.to_string().bright_black())?;
                }
            }
        }
        Ok(())
    }
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
        for ch in word.chars() {
            ret.letters.push((ch, State::Absent));
        }
        ret
    }

    fn is_solved(&self) -> bool {
        if self.letters.is_empty() {
            return false;
        }

        for (_, state) in &self.letters {
            if *state != State::Correct {
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
    solved: bool,
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
            solved: false,
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

    fn parse_input(&self, input: &str) -> Result<Word> {
        let trimmed_input = input.trim();

        if !trimmed_input.is_ascii() {
            return Err(anyhow!("not ascii"));
        }

        if trimmed_input.len() != WORD_LEN {
            return Err(anyhow!("incorrect word length"));
        }

        if !self
            .valid_words
            .contains(&trimmed_input.to_ascii_uppercase())
        {
            return Err(anyhow!("invalid word"));
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
        for (i, (c, state)) in user_input.letters.iter_mut().enumerate() {
            if answer_vec[i] == *c {
                *state = State::Correct;
                if let Some(val) = answer_map.get_mut(c) {
                    *val -= 1;
                }
            }
        }

        // check present and absent letters
        for (c, state) in user_input.letters.iter_mut() {
            if *state == State::Correct {
                continue;
            }

            match answer_map.get_mut(c) {
                Some(val) if *val > 0 => {
                    *state = State::Present;
                    *val -= 1;
                }
                _ => *state = State::Absent,
            }
        }
    }

    fn update_status(&mut self, result: &Word) {
        self.history.push(result.clone());
        let mut solved: bool = true;

        // update keyboard
        for (c, state) in result.letters.iter() {
            if *state != State::Correct {
                solved = false;
            }

            let used_state = self.used_chars.entry(*c).or_insert(State::Unused);
            if *used_state == State::Unused
                || (*used_state == State::Present && *state == State::Correct)
            {
                *used_state = *state;
            }
        }

        // update status
        self.solved = solved;
    }

    fn update_screen(&self) {
        // show previous guesses
        for word in &self.history {
            println!("{}", word);
        }

        let keyboard_layout = ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];
        for (row, chars) in keyboard_layout.iter().enumerate() {
            for _ in 0..row {
                print!(" ");
            }
            for ch in chars.to_string().chars() {
                let state = self.used_chars.get(&ch).unwrap_or(&State::Unused);
                match state {
                    State::Correct => print!("{} ", ch.to_string().green()),
                    State::Present => print!("{} ", ch.to_string().yellow()),
                    State::Absent => print!("{} ", ch.to_string().white()),
                    State::Unused => print!("{} ", ch.to_string().bright_black()),
                }
            }
            println!()
        }
        println!();
        io::stdout().flush().expect("failed to flush");
    }

    fn run(&mut self) -> Result<()> {
        while self.round <= ROUND {
            println!("ROUND {}", self.round);
            print!("> ");

            // take user input
            io::stdout().flush().expect("failed to flush");
            let mut user_input = String::new();
            io::stdin().read_line(&mut user_input)?;

            // parsing
            let mut guess = match self.parse_input(&user_input) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("{:#}", e);
                    continue;
                }
            };

            // compare
            self.compare(&mut guess);

            // update game status
            self.update_status(&guess);

            // update screen
            self.update_screen();

            self.round += 1;

            if self.solved {
                break;
            }
        }

        if self.solved {
            println!("you got it right! '{}'", self.answer);
        } else {
            println!("you lose! answer: '{}'", self.answer);
        }

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

        for (_, state) in &mut w.letters {
            *state = State::Correct;
        }
        w.letters[3].1 = State::Present;
        assert!(!w.is_solved());

        for (_, state) in &mut w.letters {
            *state = State::Correct;
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
                .map(|(_, state)| state)
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
                .map(|(_, state)| state)
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
                .map(|(_, state)| state)
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
                .map(|(_, state)| state)
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
