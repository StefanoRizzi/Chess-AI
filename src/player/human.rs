
use super::*;
use std::io::{self, Write};

pub struct HumanPlayer {
    name: String,
}

impl HumanPlayer {
    pub fn new(name: String) -> HumanPlayer {
        HumanPlayer { name }
    }
}

impl ChessPlayer for HumanPlayer {
    fn name(&self) -> &str {&self.name}
    fn make_move(&mut self, r#move: Move) {}
    fn notify_new_game(&self) {}
    fn set_position(&mut self, chess: &Chess) {}
    fn best_move(&mut self, chess: &mut Chess, time: Option<Duration>) -> Move {
        loop {
            let mut text = String::new();
            /*
            write_to_log("\nHashes:");
            for s in &chess.irreversable_state {
                write_to_log(&format!("{:?}", s.4));
            }*/
            
            write!(io::stderr(), "{}: ", self.name).unwrap(); io::stderr().flush().unwrap();
            io::stdin().read_line(&mut text).unwrap();
            write!(io::stderr(), "\r\x1b[A\x1b[J").unwrap(); io::stderr().flush().unwrap();
            
            let r#move = Move::from_text(&chess, &text.trim());
            if chess.generate_legal_moves().contains(&r#move) {
                return r#move;
            }
        }
    }
    fn evaluate_infinite(&mut self, chess: &mut Chess) -> Eval {0}
}
