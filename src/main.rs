enum State {
    Match,
    Exist,
    NotExist,
}
struct Word {
    letters: [char; 5],
    states: [State; 5],
}

struct Game {
    round: u8, // maximum 6 rounds
    answer: String,
    history: Vec<Word>,
}

impl Game {
    fn new() -> Self {
        Game {
            round: 0,
            answer: todo!(), // draw an answer
            history: Vec::new(),
        }
    }

    fn compare(self, user: String) -> Word {
        Word {
            letters: todo!(),
            states: todo!(),
        }
    }

    fn update_screen(self) {}

    fn parse_input(self, input: String) {}

    fn run(&mut self) {
        while self.round < 6 {
            // take user input
            let user_input: String = todo!();

            // parsing
            self.parse_input(user_input);

            // compare
            let result: Word = self.compare(user_input);

            // save result in history
            self.history.push(result);

            // update screen
            self.update_screen();

            self.round += 1;
        }
    }
}

fn main() {
    let mut game = Game::new();
    game.run();
}
