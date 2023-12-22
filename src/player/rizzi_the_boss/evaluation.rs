
use super::*;

const PLAWN_VALUE: Eval = 100;
const KNIGHT_VALUE: Eval = 300;
const BISHOP_VALUE: Eval = 300;
const ROOK_VALUE: Eval = 500;
const QUEEN_VALUE: Eval = 900;

impl BossPlayer {
    pub fn evaluate(&mut self, chess: &mut Chess) -> Eval {
        let white_eval = self.count_material(chess, WHITE.colour_index());
        let black_eval = self.count_material(chess, BLACK.colour_index());
    
        let mut evaluation = white_eval - black_eval;
        
        let endgame_weight = 1.0 - (white_eval + black_eval) as f32 / 7800.0;
        evaluation += self.force_king_to_corner_endgame_eval(
            chess,
            chess.side[0].king,
            chess.side[1].king,
            endgame_weight,
        );
        evaluation -= self.force_king_to_corner_endgame_eval(
            chess,
            chess.side[1].king,
            chess.side[0].king,
            endgame_weight,
        );
        
        let perspective = if chess.is_white_to_move() {1} else {-1};
        evaluation = evaluation * perspective;
        return evaluation;
    }

    pub fn count_material(&mut self, chess: &Chess, colour_index: usize) -> Eval {
        let mut material  = 0;
        material += chess.side[colour_index].pawns.len() as Eval * PLAWN_VALUE;
        material += chess.side[colour_index].knights.len() as Eval * KNIGHT_VALUE;
        material += chess.side[colour_index].bishops.len() as Eval * BISHOP_VALUE;
        material += chess.side[colour_index].rooks.len() as Eval * ROOK_VALUE;
        material += chess.side[colour_index].queens.len() as Eval * QUEEN_VALUE;
        return material;
    }

    pub fn force_king_to_corner_endgame_eval(&mut self, chess: &mut Chess, friendly_king: Square, opponent_king: Square, endgame_weight: f32) -> Eval {
        let mut evaluation = 0 as Eval;

        let opponent_rank = opponent_king / 8;
        let opponent_file = opponent_king % 8;
    
        let opponent_king_distance_file = i8::max(3 - opponent_file, opponent_file - 4);
        let opponent_king_distance_rank = i8::max(3 - opponent_rank, opponent_rank - 4);
        let opponent_king_distnce_from_centre = opponent_king_distance_file + opponent_king_distance_rank;
        evaluation += opponent_king_distnce_from_centre as Eval;
        let friendly_rank = friendly_king / 8;
        let friendly_file = friendly_king % 8;
        
        let dist_file = i8::abs(friendly_file - opponent_file);
        let dist_rank = i8::abs(friendly_rank - opponent_rank);
        let dist_between_kings = dist_file + dist_rank;
        evaluation += 14 - dist_between_kings as Eval;
        
        return ((evaluation * 10) as f32 * endgame_weight) as Eval;
    }
}

impl PieceType {
    pub fn get_piece_value(self) -> i16 {
        match self {
            PAWN => PLAWN_VALUE,
            KNIGHT => KNIGHT_VALUE,
            BISHOP => BISHOP_VALUE,
            ROOK => ROOK_VALUE,
            QUEEN => QUEEN_VALUE,
            KING => 0,
            _ => unreachable!()
        }
    }
}