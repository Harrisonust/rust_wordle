use std::collections::{HashMap, HashSet};

const ROUND: u8 = 6;
const WORD_LEN: usize = 5;

#[derive(Debug, PartialEq, Clone, Copy)]
enum State {
    Correct,
    Present,
    Absent,
    Unused,
}

#[derive(Debug, Clone, Copy)]
struct Word {
    letters: [char; WORD_LEN],
    states: [State; WORD_LEN],
}

impl Word {
    fn new() -> Self {
        todo!();
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
        let words = Wordle::load_words();
        let answer = Wordle::draw_word(&words);
        Wordle {
            round: 1,
            valid_words: words,
            used_chars: HashMap::new(),
            answer,
            history: Vec::new(),
        }
    }

    fn load_words() -> HashSet<String> {
        todo!();
    }

    fn draw_word(words: &HashSet<String>) -> String {
        todo!();
    }

    fn compare(&self, user_input: &[char; WORD_LEN]) -> Word {
        todo!();
    }

    fn update_screen(&self) {
        // show previous guesses
        todo!();

        // show keyboard (print A to Z)
        todo!();
    }

    fn parse_input(&self, input: &str) -> Result<[char; WORD_LEN], String> {
        todo!();
    }

    fn update_status(&mut self, result: &Word) {
        todo!();
    }

    fn is_solved(&self) -> bool {
        todo!();
    }

    fn run(&mut self) {
        while self.round <= ROUND {
            // take user input
            let user_input = String::new();

            // parsing
            let guess = match self.parse_input(&user_input) {
                Ok(g) => g,
                Err(e) => {
                    println!("Error: {e}");
                    continue; // restart current round
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
    }
}

fn main() {
    let mut game = Wordle::new();
    game.run();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_solved_test() {
        let mut w = Word::new();
        assert!(!w.is_solved());

        for state in &mut w.states {
            *state = State::Correct;
        }
        w.states[3] = State::Present;
        assert!(!w.is_solved());

        for state in &mut w.states {
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
        let result = Wordle::draw_word(&words);
        assert!(words.contains(&result));
    }

    #[test]
    fn load_words_test() {
        let words = Wordle::load_words();
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
        let w = game.compare(&['C', 'A', 'T', 'E', 'R']);
        assert!(w.states[0] == State::Correct);
        assert!(w.states[1] == State::Present);
        assert!(w.states[2] == State::Present);
        assert!(w.states[3] == State::Present);
        assert!(w.states[4] == State::Present);

        let mut game = Wordle::new();
        game.answer = "HOUND".to_string();
        let w = game.compare(&['A', 'M', 'O', 'N', 'G']);
        assert!(w.states[0] == State::Absent);
        assert!(w.states[1] == State::Absent);
        assert!(w.states[2] == State::Present);
        assert!(w.states[3] == State::Correct);
        assert!(w.states[4] == State::Absent);
    }

    #[test]
    fn update_status_test() {
        let mut game = Wordle::new();
        game.answer = "DEALT".to_string();
        let w = game.compare(&['A', 'S', 'I', 'D', 'E']);
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

        let w = game.compare(&['D', 'E', 'A', 'T', 'H']);
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

        let w = game.compare(&['D', 'E', 'A', 'L', 'T']);
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
