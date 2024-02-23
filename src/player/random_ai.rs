
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
    fn best_move(&mut self,  chess: &mut Chess, time: Option<Duration>) -> (Move, Eval) {
        let legal_moves = chess.generate_legal_moves();
        return (legal_moves[self.rng.gen_range(0..legal_moves.len())], 0);
    }
    fn make_move(&mut self, r#move: Move) {}
    fn evaluate_infinite(&mut self,
        chess: &mut Chess,
        send_info: fn(depth: u16, eval: Eval, time: u64, nodes: u32, nps: u32, pv: Move),
    ) {}
}