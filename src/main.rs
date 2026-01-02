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
    Default,
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
                    write!(f, "{}", c.to_string().blue())?;
                }
                State::Present => {
                    write!(f, "{}", c.to_string().yellow())?;
                }
                State::Unused => {
                    write!(f, "{}", c.to_string().purple())?;
                }
                State::Default => {}
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

        let mut solved: bool = true;
        for (c, state) in &self.letters {
            if *state != State::Correct {
                solved = false;
            }
        }
        solved
    }
}

struct Wordle {
    round: u8, // maximum 6 rounds
    valid_words: HashSet<String>,
    used_chars: HashMap<char, State>,
    answer: String,
    history: Vec<Word>,
    status: bool,
}

impl Wordle {
    fn new() -> Self {
        let valid_words = Wordle::load_words().expect("failed to load words");

        let answer = Wordle::draw_word(&valid_words).expect("failed to draw word");

        let mut used_chars = HashMap::new();
        for i in 65u8..=90 {
            used_chars.entry(i as char).or_insert(State::Unused);
        }

        Wordle {
            round: 1,
            valid_words,
            used_chars,
            answer,
            history: Vec::new(),
            status: false,
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
        words.into_iter().choose(&mut rng).cloned()
    }

    fn parse_input(&self, input: &str) -> Result<Word> {
        let trimmed_input = input.trim_end();

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

    fn compare(&self, user_input: &Word) -> Word {
        let mut input_map: HashMap<char, u8> = HashMap::new();
        let mut answer_map: HashMap<char, u8> = HashMap::new();

        self.answer.chars().for_each(|c| {
            let count = answer_map.entry(c).or_insert(0);
            *count += 1;
        });

        let answer_vec: Vec<char> = self.answer.chars().collect();
        let mut letters = user_input.clone().letters;

        // check correct letters
        for (i, (c, state)) in letters.iter_mut().enumerate() {
            if answer_vec[i] == *c {
                *state = State::Correct;
                *input_map.entry(*c).or_insert(0) += 1;
            }
        }

        // check present and absent letters
        for (c, state) in letters.iter_mut() {
            if *state == State::Correct {
                continue;
            }

            match answer_map.get(c) {
                Some(&val) => {
                    let count = input_map.entry(*c).or_insert(0);
                    *count += 1;

                    if *count <= val {
                        *state = State::Present;
                    } else {
                        *state = State::Absent;
                    }
                }
                None => {
                    *state = State::Absent;
                }
            }
        }

        Word { letters }
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

            if *used_state == State::Unused {
                *used_state = *state;
            } else if *used_state == State::Present && *state == State::Correct {
                *used_state = *state;
            }
        }

        // update status
        self.status = solved;
    }

    fn update_screen(&self) {
        // show previous guesses
        for word in &self.history {
            println!("{}", word);
        }

        // show keyboard (print A to Z)
        for i in 65u8..=90 {
            let c = i as char;
            match self.used_chars[&c] {
                State::Correct => {
                    print!("{}", c.to_string().green());
                }
                State::Absent => {
                    print!("{}", c.to_string().blue());
                }
                State::Present => {
                    print!("{}", c.to_string().yellow());
                }
                State::Unused => {
                    print!("{}", c.to_string().purple());
                }
                State::Default => {}
            }
        }
        println!("");
        io::stdout().flush().expect("failed to flush");
    }

    fn is_solved(&self) -> bool {
        self.status
    }

    fn run(&mut self) -> Result<()> {
        while self.round <= ROUND {
            // take user input
            println!("ROUND {}", self.round);
            print!("> ");
            io::stdout().flush().expect("failed to flush");
            let mut user_input = String::new();
            let _ = io::stdin().read_line(&mut user_input);

            // parsing
            let guess = match self.parse_input(&user_input) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("{:#}", e);
                    continue;
                }
            };

            // compare
            let result: Word = self.compare(&guess);

            // update game status
            self.update_status(&result);

            // update screen
            self.update_screen();

            self.round += 1;

            if self.is_solved() {
                break;
            }
        }

        if self.is_solved() {
            println!("you got it right! '{}'", self.answer);
        } else {
            println!("you lose! answer: '{}'", self.answer);
        }

        Ok(())
    }
}

fn main() {
    let mut game = Wordle::new();

    println!("log answer here: {}", game.answer);

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
    fn compare_test_1() {
        let mut game = Wordle::new();
        game.answer = "CRATE".to_string();
        let w = game.compare(&Word::from("CATER"));
        assert_eq!(
            w.letters,
            vec![
                ('C', State::Correct),
                ('A', State::Present),
                ('T', State::Present),
                ('E', State::Present),
                ('R', State::Present)
            ]
        );

        let mut game = Wordle::new();
        game.answer = "HOUND".to_string();
        let w = game.compare(&Word::from("AMONG"));
        assert_eq!(
            w.letters,
            [
                ('A', State::Absent),
                ('M', State::Absent),
                ('O', State::Present),
                ('N', State::Correct),
                ('G', State::Absent)
            ]
        );

        let mut game = Wordle::new();
        game.answer = "TRAIT".to_string();
        let w = game.compare(&Word::from("TXTXT"));
        assert_eq!(
            w.letters.iter().map(|x| x.1).collect::<Vec<State>>(),
            [
                State::Correct,
                State::Absent,
                State::Absent,
                State::Absent,
                State::Correct,
            ]
        );
    }

    #[test]
    fn compare_test_2() {
        let mut game = Wordle::new();
        game.answer = "CRATE".to_string();
        let w = game.compare(&Word::from("CATER"));
        assert_eq!(
            w.letters.iter().map(|x| x.1).collect::<Vec<State>>(),
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
        let w = game.compare(&Word::from("AMONG"));
        assert_eq!(
            w.letters.iter().map(|x| x.1).collect::<Vec<State>>(),
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
        let w = game.compare(&Word::from("TXTTX"));
        assert_eq!(
            w.letters.iter().map(|x| x.1).collect::<Vec<State>>(),
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
        let w = game.compare(&Word::from("ASIDE"));
        game.update_status(&w);
        for (&ch, &state) in &game.used_chars {
            if ch == 'A' || ch == 'D' || ch == 'E' {
                assert!(state == State::Present);
            } else if ch == 'S' || ch == 'I' {
                assert!(state == State::Absent);
            } else {
                assert!(state == State::Unused);
            }
        }
        assert!(!game.is_solved());

        let w = game.compare(&Word::from("DEATH"));
        game.update_status(&w);
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
        assert!(!game.is_solved());

        let w = game.compare(&Word::from("DEALT"));

        game.update_status(&w);
        for (&ch, &state) in &game.used_chars {
            if ch == 'D' || ch == 'E' || ch == 'A' || ch == 'L' || ch == 'T' {
                assert!(state == State::Correct);
            } else if ch == 'H' || ch == 'S' || ch == 'I' {
                assert!(state == State::Absent);
            } else {
                assert!(state == State::Unused);
            }
        }
        assert!(game.is_solved());
    }
}
