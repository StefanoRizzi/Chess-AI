use std::cmp::min;

use super::*;

pub const DIRECTION_OFFSETS: [i8; 8] = [8,-8,1,-1,7,-7,9,-9];
pub static mut NUM_SQUARES_TO_EDGES: [[u8; 8]; 64] = [[0; 8]; 64];

pub fn precomputed_move_data() {
    for rank in 0..8 {
        for file in 0..8 {
            let num_north = 7 - rank;
            let num_south = rank;
            let num_east = 7 - file;
            let num_west = file;
            unsafe {
                NUM_SQUARES_TO_EDGES[(rank * 8 + file) as usize] = [
                    num_north,
                    num_south,
                    num_east,
                    num_west,
                    min(num_north, num_west),
                    min(num_south, num_east),
                    min(num_north, num_east),
                    min(num_south, num_west),
                    ];
            }
        }
    }
}

impl Chess {
    pub fn generate_legal_moves(&mut self) -> Vec<Move> {
        let colour = self.colour_to_move();
        let mut legal_moves = self.generate_moves();
        legal_moves.retain(|move_to_verify| {
            self.make_move(move_to_verify);
            
            let keep = self.generate_moves().iter().all(|response| self.board[response.target_square as usize] != piece::KING | colour);

            self.unmake_move(move_to_verify);
            keep
        });
        legal_moves
    }

    fn generate_moves(&mut self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        for start_square in 0..64 {
            let piece = self.board[start_square as usize];
            if piece::is_colour(piece, self.colour_to_move()) {
                if piece::is_sliding_piece(piece) {
                    self.generate_sliding_moves(&mut moves, start_square, piece);
                }
                else if piece::is_type(piece, piece::PAWN) {
                    self.generate_pawn_moves(&mut moves, start_square, piece);
                }
                else if piece::is_type(piece, piece::KNIGHT) {
                    self.generate_knight_moves(&mut moves, start_square, piece);
                }
                else {
                    self.generate_king_moves(&mut moves, start_square, piece);
                }
            }
        }
        moves
    }

    fn generate_sliding_moves(&mut self, moves: &mut Vec<Move>, start_square: u8, piece: u8) {
        let start_dir_index = if piece::is_type(piece, BISHOP) {4} else {0};
        let end_dir_index = if piece::is_type(piece, ROCK) {4} else {8};
        
        for dir_index in start_dir_index..end_dir_index {
            let num_squares_to_edges = unsafe {NUM_SQUARES_TO_EDGES[start_square as usize][dir_index]};
            for n in 0..num_squares_to_edges {
                let target_square = (start_square as i8 + DIRECTION_OFFSETS[dir_index] * (n + 1) as i8) as u8;
                let piece_on_target_square = self.board[target_square as usize];

                if piece::is_colour(piece_on_target_square, self.colour_to_move()) {break}

                moves.push(Move { start_square, target_square, promotion: piece::NONE });

                if piece_on_target_square != piece::NONE {break}
            }
        }
    }

    fn generate_pawn_moves(&mut self, moves: &mut Vec<Move>, start_square: u8, piece: u8) {
        let dir_indices = if self.white_turn {[0,4,6]} else {[1,5,7]};
        for &dir_index in &dir_indices[1..3] {
            let num_squares_to_edges = unsafe {NUM_SQUARES_TO_EDGES[start_square as usize][dir_index]};
            if num_squares_to_edges > 0 {
                let target_square = (start_square as i8 + DIRECTION_OFFSETS[dir_index]) as u8;
                if piece::is_colour(self.board[target_square as usize], self.colour_opponent()) || target_square == self.en_passant {
                    let num_squares_to_edges = unsafe {NUM_SQUARES_TO_EDGES[start_square as usize][dir_indices[0]]};
                    if num_squares_to_edges == 1 {
                        moves.push(Move { start_square, target_square, promotion: QUEEN });
                        moves.push(Move { start_square, target_square, promotion: BISHOP });
                        moves.push(Move { start_square, target_square, promotion: ROCK });
                        moves.push(Move { start_square, target_square, promotion: KNIGHT });
                    } else {
                        moves.push(Move { start_square, target_square, promotion: NONE });
                    }    
                }
            }
        }
        let mut target_square = (start_square as i8 + DIRECTION_OFFSETS[dir_indices[0]]) as u8;
        if self.board[target_square as usize] == NONE {
            let num_squares_to_edges = unsafe {NUM_SQUARES_TO_EDGES[start_square as usize][dir_indices[0]]};
            if num_squares_to_edges == 1 {
                moves.push(Move { start_square, target_square, promotion: QUEEN });
                moves.push(Move { start_square, target_square, promotion: BISHOP });
                moves.push(Move { start_square, target_square, promotion: ROCK });
                moves.push(Move { start_square, target_square, promotion: KNIGHT });
            } else {
                moves.push(Move { start_square, target_square, promotion: NONE });
                if num_squares_to_edges == 6 {
                    target_square = (target_square as i8 + DIRECTION_OFFSETS[dir_indices[0]]) as u8;
                    if self.board[target_square as usize] == NONE {
                        moves.push(Move { start_square, target_square, promotion: NONE });
                    }
                }
            }
        }
    }
    
    fn generate_knight_moves(&mut self, moves: &mut Vec<Move>, start_square: u8, piece: u8) {
        for dir in [-17, -15, -10, -6, 6, 10, 15, 17] {
            let target_square = (start_square as i8 + dir) as u8;
            if target_square >= 64 {continue}
            if start_square as i8 % 8 - target_square as i8 % 8 > 2 {continue}
            if start_square as i8 % 8 - target_square as i8 % 8 < -2 {continue}
            if piece::is_colour(self.board[target_square as usize], self.colour_to_move()) {continue}

            moves.push(Move { start_square, target_square, promotion: NONE })
        }
    }
    
    fn generate_king_moves(&mut self, moves: &mut Vec<Move>, start_square: u8, piece: u8) {
        for dir_index in 0..8 {
            let num_squares_to_edges = unsafe {NUM_SQUARES_TO_EDGES[start_square as usize][dir_index]};
            if num_squares_to_edges >= 1 {
                let target_square = (start_square as i8 + DIRECTION_OFFSETS[dir_index]) as u8;
                if !piece::is_colour(self.board[target_square as usize], self.colour_to_move()) {
                    moves.push(Move { start_square, target_square, promotion: NONE });
                }
            }
        }
        if ( self.white_turn && (self.castle & castle::CASTLE_W_K != 0)) 
        || (!self.white_turn && (self.castle & castle::CASTLE_B_K != 0)) {
            if self.board[(start_square + 1) as usize] == NONE
            && self.board[(start_square + 2) as usize] == NONE {
                moves.push(Move { start_square, target_square: start_square + 2, promotion: NONE });
            }
        }
        if ( self.white_turn && (self.castle & castle::CASTLE_W_Q != 0)) 
        || (!self.white_turn && (self.castle & castle::CASTLE_B_Q != 0)) {
            if self.board[(start_square - 1) as usize] == NONE
            && self.board[(start_square - 2) as usize] == NONE 
            && self.board[(start_square - 3) as usize] == NONE {
                moves.push(Move { start_square, target_square: start_square - 2, promotion: NONE });
            }
        }
    }
}