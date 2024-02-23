
use crate::legal_moves::piece_attacks;

use super::*;

pub mod square_table;
pub use square_table::*;

const PLAWN_VALUE: Eval = 100;
const KNIGHT_VALUE: Eval = 300;
const BISHOP_VALUE: Eval = 300;
const ROOK_VALUE: Eval = 500;
const QUEEN_VALUE: Eval = 900;

impl BossPlayer {
    pub fn evaluate(&mut self, chess: &mut Chess) -> Eval {
        self.evaluated += 1;

        let white_material = self.count_material(chess, WHITE.colour_index());
        let black_material = self.count_material(chess, BLACK.colour_index());
        
        let opening_weight = white_material.min(black_material) as f32 / 3900.0;
        let endgame_weight = 1.0 - opening_weight;

        let mut white_eval = endgame_weight * white_material as f32;
        let mut black_eval = endgame_weight * black_material as f32;
        
        white_eval += opening_weight * self.count_opening_map_material(chess, WHITE) as f32;
        black_eval += opening_weight * self.count_opening_map_material(chess, BLACK) as f32;
        
        white_eval += endgame_weight * self.count_endgame_map_material(chess, WHITE.colour_index()) as f32;
        black_eval += endgame_weight * self.count_endgame_map_material(chess, BLACK.colour_index()) as f32;
        
        //white_eval += opening_weight * self.count_square_control(chess, WHITE.colour_index()) as f32;
        //black_eval += opening_weight * self.count_square_control(chess, BLACK.colour_index()) as f32;

        white_eval += opening_weight * self.king_security(chess, WHITE) as f32;
        black_eval += opening_weight * self.king_security(chess, BLACK) as f32;

        white_eval += endgame_weight * self.force_king_to_corner_endgame_eval(chess, WHITE) as f32;
        black_eval += endgame_weight * self.force_king_to_corner_endgame_eval(chess, BLACK) as f32;

        let mut evaluation = (white_eval - black_eval) as Eval;
        evaluation += (100.0 * endgame_weight) as Eval;

        return if chess.is_white_to_move() {evaluation} else {-evaluation};
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
    pub fn count_opening_map_material(&mut self, chess: &Chess, colour: Colour) -> Eval {
        let mut material  = 0;
        let mut piece_attacked = 0;
        let colour_index = colour.colour_index();
        let enemy_attacks = &chess.side[colour.opponent().colour_index()].attacks;

        //let pawn_defenses = chess.side[colour_index].piece_attacks[PAWN.piece_index()];
        let mut pawn = [false; 8];
        for &square in &chess.side[colour_index].pawns {
            let square = if colour_index == BLACK.colour_index() {square} else {(7-square/8)*8 + square%8};
            material += opening::PAWN_SQAURE_TABLE[square as usize];
            if enemy_attacks[square as usize] > 0 {piece_attacked += 1}
            //material += 10 * pawn_defenses[square as usize] as Eval;
            if pawn[(square%8) as usize] {material -= 20}
            pawn[(square%8) as usize] = true;
            
        }
        let mut len_wall = 0;
        for (col, &pawn) in pawn.iter().enumerate() {
            if !pawn && len_wall == 1 {
                len_wall = 0;
                material -= if col == 1 {50} else {60};
            }
            if pawn {len_wall += 1}
        }
        if len_wall == 1 {
            material -= 50;
        }

        for &square in &chess.side[colour_index].knights {
            let square = if colour_index == BLACK.colour_index() {square} else {(7-square/8)*8 + square%8};
            material += opening::KNIGHT_SQAURE_TABLE[square as usize];
            if enemy_attacks[square as usize] > 0 {piece_attacked += 1}
        }
        for &square in &chess.side[colour_index].bishops {
            let square = if colour_index == BLACK.colour_index() {square} else {(7-square/8)*8 + square%8};
            material += opening::BISHOP_SQAURE_TABLE[square as usize];
            if enemy_attacks[square as usize] > 0 {piece_attacked += 1}
        }
        for &square in &chess.side[colour_index].rooks {
            let square = if colour_index == BLACK.colour_index() {square} else {(7-square/8)*8 + square%8};
            material += opening::ROOK_SQAURE_TABLE[square as usize];
            if enemy_attacks[square as usize] > 0 {piece_attacked += 1}
        }
        for &square in &chess.side[colour_index].queens {
            let square = if colour_index == BLACK.colour_index() {square} else {(7-square/8)*8 + square%8};
            material += opening::QUEEN_SQAURE_TABLE[square as usize];
            if enemy_attacks[square as usize] > 0 {piece_attacked += 1}
        }
        let square = chess.side[colour_index].king;
        let square = if colour_index == BLACK.colour_index() {square} else {(7-square/8)*8 + square%8};
        material += opening::KING_SQAURE_TABLE[square as usize];
        if enemy_attacks[square as usize] > 0 {piece_attacked += 1}

        material += 160 - 10 * piece_attacked;

        return material;
    }
    pub fn count_endgame_map_material(&mut self, chess: &Chess, colour_index: usize) -> Eval {
        let mut material  = 0;
        let pawn_defenses = chess.side[colour_index].piece_attacks[PAWN.piece_index()];

        for &square in &chess.side[colour_index].pawns {
            let square = if colour_index == BLACK.colour_index() {square} else {(7-square/8)*8 + square%8};
            material += endgame::PAWN_SQAURE_TABLE[square as usize];
            material += 10 * pawn_defenses[square as usize] as Eval;
        }
        return material;
    }
    pub fn count_square_control(&mut self, chess: &Chess, colour_index: usize) -> Eval {
        return 1 * chess.side[colour_index].attacks.iter().sum::<i8>() as Eval;
    }
    pub fn king_security(&mut self, chess: &Chess, colour: Colour) -> Eval {
        let king = chess.side[colour.colour_index()].king;
        let mut enemy_control = 0;
        let mut defense_piece = 0;
        let mut defense_control = 0;
        let king_squares = piece_attacks(KING, colour, king);
        let num_squares = king_squares.len();
        for &mut square in king_squares {
            enemy_control += chess.side[colour.opponent().colour_index()].attacks[square as usize];
            if chess.board(square) != NONE { defense_piece += 1 }
            defense_control += chess.side[colour.colour_index()].attacks[square as usize] - 1;
        }
        let mut evaluation = 530;
        evaluation -= 500.min(10 * enemy_control as Eval);
        evaluation += 10 * 1.min(defense_piece as Eval);
        evaluation += 5 * 4.min(defense_control as Eval);
        if num_squares == defense_piece {evaluation -= 30}
        return evaluation;
    }

    pub fn force_king_to_corner_endgame_eval(&mut self, chess: &mut Chess, colour: Colour) -> Eval {
        let mut evaluation = 0 as Eval;

        let friendly_king = chess.side[colour.colour_index()].king;
        let opponent_king = chess.side[colour.opponent().colour_index()].king;

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
        
        return evaluation * 10;
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
