const ROUND: u8 = 6;
const WORD_LEN: usize = 5;

#[derive(Clone)]
enum State {
    Match,
    Exist,
    NotExist,
}

#[derive(Clone)]
struct Word {
    letters: String, // make sure this has only 5 characters
    states: [State; WORD_LEN],
}

struct Game {
    round: u8, // maximum 6 rounds
    valid_words: Vec<String>,
    answer: String,
    history: Vec<Word>,
}

impl Game {
    fn new() -> Self {
        let words = Game::load_words();
        let answer = Game::draw_word(&words);

        Game {
            round: 1,
            valid_words: words,
            answer,
            history: Vec::new(),
        }
    }

    fn load_words() -> Vec<String> {
        todo!();
    }

    fn draw_word(words: &[String]) -> String {
        todo!();
    }

    fn compare(&self, user: &String) -> Word {
        todo!();
    }

    fn update_screen(&self) {
        todo!();
    }

    fn parse_input(&self, input: &str) -> Result<String, &str> {
        todo!();
    }

    fn is_correct(&self, guess: &Word) -> bool {
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

            // save result in history
            self.history.push(result.clone());

            // update screen
            self.update_screen();

            self.round += 1;

            if self.is_correct(&result) {
                break;
            }
        }
    }
}

fn main() {
    let mut game = Game::new();
    game.run();
}
