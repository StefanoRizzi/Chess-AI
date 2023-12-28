
use rand::rngs::ThreadRng;
use rand::Rng;

use super::*;

pub struct BadPlayer {
    rng: ThreadRng,
}
impl BadPlayer {
    pub fn new() -> BadPlayer {
        BadPlayer { rng: rand::thread_rng() }
    }
}

impl ChessPlayer for BadPlayer {
    fn name(&self) -> &str {"Random Player"}
    fn notify_new_game(&self) {}
    fn set_position(&mut self, chess: &Chess) {}
    fn best_move(&mut self,  chess: &mut Chess, time: Option<Duration>) -> Move {
        let legal_moves = chess.generate_legal_moves();
        return legal_moves[self.rng.gen_range(0..legal_moves.len())];
    }
    fn make_move(&mut self, r#move: Move) {}
    fn evaluate_infinite(&mut self, chess: &mut Chess) -> Eval {0}
}