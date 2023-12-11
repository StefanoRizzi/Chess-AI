

use rand::rngs::ThreadRng;

use super::*;



pub struct RizziPlayer {
    rng: ThreadRng,
}
impl RizziPlayer {
    fn evaluation_ab(&mut self, chess: &mut Chess, depth: i16, alpha: f32, beta: f32) -> f32 {
        if depth == 0 {
            return self.evaluation_static(chess);
        }
        return 0.0;
    }

    fn evaluation_static(&mut self, chess: &mut Chess) -> f32 {
        return 0.0;
    }
}

impl ChessPlayer for RizziPlayer {
    fn best_move(&mut self, chess: &mut Chess) -> Move {
        todo!()    
    }
}