
use super::*;

impl PieceType {
    pub fn getValue(self) -> u16 {
        match self {
            PAWN => 100,
            KNIGHT => 300,
            BISHOP => 300,
            ROOK => 500,
            QUEEN => 900,
            _ => unreachable!(),
        }
    }
}

pub struct RizziPlayer {

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

    fn material_vale(&self, chess: &mut Chess) -> u16 {
        todo!()
    }
}

impl ChessPlayer for RizziPlayer {
    fn best_move(&mut self, chess: &mut Chess) -> Moves {
        todo!()    
    }
}