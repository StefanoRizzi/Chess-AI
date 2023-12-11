
use rand::rngs::ThreadRng;

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
    fn best_move(&mut self, chess: &mut Chess) -> Move {
        let legal_moves = chess.generate_legal_moves();
        return legal_moves[self.rng.gen_range(0..legal_moves.len())];
    }
}