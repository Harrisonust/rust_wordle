use anyhow::{Result, anyhow};
use core::panic;
use rand::seq::IteratorRandom;
use std::collections::{HashMap, HashSet};
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
        todo!();
    }
}

struct Wordle {
    round: u8, // maximum 6 rounds
    valid_words: HashSet<String>,
    used_chars: HashMap<char, State>,
    answer: String,
    history: Vec<Word>,
}

impl Wordle {
    fn new() -> Self {
        let words = Wordle::load_words().expect("failed to load words");

        let answer = Wordle::draw_word(&words).expect("failed to draw word");

        Wordle {
            round: 1,
            valid_words: words,
            used_chars: HashMap::new(),
            answer,
            history: Vec::new(),
        }
    }

    fn load_words() -> Result<HashSet<String>> {
        let file = File::open(FILE_PATH)?;
        let reader: BufReader<File> = BufReader::new(file);

        let mut string_set = HashSet::new();

        for line in reader.lines() {
            let line = line?;
            string_set.insert(line.to_ascii_uppercase());
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

    fn parse_input(&self, input: &str) -> Result<Vec<(char, State)>> {
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

        Ok(trimmed_input
            .chars()
            .map(|c| (c.to_ascii_uppercase(), State::Default))
            .collect())
    }

    fn compare(&self, user_input: &Vec<(char, State)>) -> Word {
        let mut input_map: HashMap<char, u8> = HashMap::new();
        let mut answer_map: HashMap<char, u8> = HashMap::new();

        self.answer.chars().for_each(|c| {
            let count = answer_map.entry(c).or_insert(0);
            *count += 1;
        });

        let answer_vec: Vec<char> = self.answer.chars().collect();
        let mut letters = user_input.clone();

        // check correct letters
        for (i, (c, state)) in letters.iter_mut().enumerate() {
            if answer_vec[i] == *c {
                *state = State::Correct;
                *input_map.entry(*c).or_insert(0) += 1;
            } else if !answer_map.get(c).is_some() {
                *state = State::Absent;
            }
        }

        for (c, state) in letters.iter_mut() {
            if *state != State::Default {
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
        todo!();
    }

    fn update_screen(&self) {
        // show previous guesses
        todo!();

        // show keyboard (print A to Z)
        todo!();
    }

    fn is_solved(&self) -> bool {
        todo!();
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

            println!("{:?}", guess);

            // compare
            let result: Word = self.compare(&guess);

            println!("{:?}", result);

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
        let mut w = Word::new();
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
        let w = game.compare(&vec![
            ('C', State::Default),
            ('A', State::Default),
            ('T', State::Default),
            ('E', State::Default),
            ('R', State::Default),
        ]);
        // TODO: change to game.compare(Word::from("CATER"));
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
        let w = game.compare(&vec![
            ('A', State::Default),
            ('M', State::Default),
            ('O', State::Default),
            ('N', State::Default),
            ('G', State::Default),
        ]);
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
        let w = game.compare(&vec![
            ('T', State::Default),
            ('X', State::Default),
            ('T', State::Default),
            ('X', State::Default),
            ('T', State::Default),
        ]);
        assert_eq!(
            w.letters,
            [
                ('T', State::Correct),
                ('X', State::Absent),
                ('T', State::Absent),
                ('X', State::Absent),
                ('T', State::Correct)
            ]
        );
    }

    #[test]
    fn compare_test_2() {
        let mut game = Wordle::new();
        game.answer = "CRATE".to_string();
        let w = game.compare(&vec![
            ('C', State::Default),
            ('A', State::Default),
            ('T', State::Default),
            ('E', State::Default),
            ('R', State::Default),
        ]);
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
        let w = game.compare(&vec![
            ('A', State::Default),
            ('M', State::Default),
            ('O', State::Default),
            ('N', State::Default),
            ('G', State::Default),
        ]);
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
        let w = game.compare(&vec![
            ('T', State::Default),
            ('X', State::Default),
            ('T', State::Default),
            ('T', State::Default),
            ('X', State::Default),
        ]);
        assert_eq!(
            w.letters,
            [
                ('T', State::Correct),
                ('X', State::Absent),
                ('T', State::Present),
                ('T', State::Absent),
                ('X', State::Absent)
            ]
        );
    }

    #[test]
    fn update_status_test() {
        let mut game = Wordle::new();
        game.answer = "DEALT".to_string();
        let w = game.compare(&vec![
            ('A', State::Default),
            ('S', State::Default),
            ('I', State::Default),
            ('D', State::Default),
            ('E', State::Default),
        ]);
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

        let w = game.compare(&vec![
            ('D', State::Default),
            ('E', State::Default),
            ('A', State::Default),
            ('T', State::Default),
            ('H', State::Default),
        ]);
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

        let w = game.compare(&vec![
            ('D', State::Default),
            ('E', State::Default),
            ('A', State::Default),
            ('L', State::Default),
            ('T', State::Default),
        ]);

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
