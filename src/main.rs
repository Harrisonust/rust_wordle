mod utils;
use utils::wordle::Wordle;

fn main() {
    if let Err(e) = Wordle::new().run() {
        eprintln!("{:#}", e);
    }
}
