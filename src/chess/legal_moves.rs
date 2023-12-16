use std::cmp::min;

use super::*;

pub const DIRECTION_OFFSETS: [i8; 8] = [8,-8,1,-1,7,-7,9,-9];
pub const DIRECTION_OPPOST_INDEX: [usize; 8] = [1,0,3,2,5,4,7,6];
pub static mut NUM_SQUARES_TO_EDGES: [[u8; 8]; 64] = [[0; 8]; 64];

pub static mut PRECOMPUTED: bool = false;
const EMPTY_VEC: Vec<u8> = Vec::new();
pub static mut KING_CONTROL: [Vec<u8>;64] = [EMPTY_VEC;64];
pub static mut KNIGHT_CONTROL: [Vec<u8>;64] = [EMPTY_VEC;64];
pub static mut WHITE_PAWN_CONTROL: [Vec<u8>;64] = [EMPTY_VEC;64];
pub static mut BLACK_PAWN_CONTROL: [Vec<u8>;64] = [EMPTY_VEC;64];

pub fn precompute() {
    if unsafe {PRECOMPUTED} {return}
    unsafe {PRECOMPUTED = true}
    precomputed_move_data();
    precomputed_king_control();
    precomputed_knight_control();
    precomputed_pawn_control();
}
fn precomputed_move_data() {
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
fn precomputed_king_control() {
    for square in 0..64 {
        for dir_index in 0..8 {
            let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[square as usize][dir_index]};
            if dist_edge >= 1 {
                let target = (square as i8 + DIRECTION_OFFSETS[dir_index]) as u8;
                unsafe {KING_CONTROL[square as usize].push(target)};
            }
        }
    }
}
fn precomputed_knight_control() {
    for square in 0..64 {
        for jump in [-17, -15, -10, -6, 6, 10, 15, 17] {
            let target = (square as i8 + jump) as u8;
            if target >= 64 {continue}
            if square as i8 % 8 - target as i8 % 8 > 2 {continue}
            if square as i8 % 8 - target as i8 % 8 < -2 {continue}
            unsafe {KNIGHT_CONTROL[square as usize].push(target)};
        }
    }
}
fn precomputed_pawn_control() {
    for square in 0..64 {
        for dir_index in [4, 6] {
            let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[square as usize][dir_index]};
            if dist_edge >= 1 {
                let target = (square as i8 + DIRECTION_OFFSETS[dir_index]) as u8;
                unsafe {WHITE_PAWN_CONTROL[square as usize].push(target)};
            }
        }
        for dir_index in [5, 7] {
            let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[square as usize][dir_index]};
            if dist_edge >= 1 {
                let target = (square as i8 + DIRECTION_OFFSETS[dir_index]) as u8;
                unsafe {BLACK_PAWN_CONTROL[square as usize].push(target)};
            }
        }
    }
}

impl Chess {
    pub fn generate_legal_moves(&self) -> Vec<Moves> {
        let (side, opponent_side) = if self.colour_turn == WHITE 
        {(&self.white_side, &self.black_side)} else
        {(&self.black_side, &self.white_side)};
        
        let (pins, attack, en_passant_able) = self.get_pins_and_slide_attack(side.king_square, self.colour_turn, side, opponent_side);
        let mut moves = Vec::new();
        
        if opponent_side.control[side.king_square as usize] == 0 {
            for &start in &side.pieces {
                let piece = self.board[start as usize].get_type();

                if let Some((_, defend_squares)) = pins.iter().find(|&piece| piece.0 == start) {
                    match piece {
                        QUEEN | ROOK | BISHOP => self.generate_sliding_defenses(&mut moves, start, piece, defend_squares),
                        KNIGHT => self.generate_knight_defenses(&mut moves, start, defend_squares),
                        PAWN if self.colour_turn == WHITE => self.generate_white_pawn_moves(&mut moves, start, en_passant_able, false, Some(defend_squares)),
                        _ => self.generate_black_pawn_moves(&mut moves, start, en_passant_able, false, Some(defend_squares)),
                    }
                }
                else {
                    match piece {
                        QUEEN | ROOK | BISHOP => self.generate_sliding_moves(&mut moves, start, piece),
                        KNIGHT => self.generate_knight_moves(&mut moves, start),
                        PAWN if self.colour_turn == WHITE => self.generate_white_pawn_moves(&mut moves, start, en_passant_able, false, None),
                        _ => self.generate_black_pawn_moves(&mut moves, start, en_passant_able, false, None),
                    }
                }
            }
        }
        else {
            if opponent_side.control[side.king_square as usize] == 1 {
                let mut is_pawn_check = false;
                let defend_squares = if let Some(attack) = attack
                {attack}
                else
                {
                    let mut defend = self.find_knight_check(&mut moves, side.king_square);
                    if defend == u8::MAX {
                        is_pawn_check = true;
                        defend = if self.colour_turn == WHITE
                        {self.find_black_pawn_check(&mut moves, side.king_square)}
                        else 
                        {self.find_white_pawn_check(&mut moves, side.king_square)};
                    }
                    Vec::from([defend])
                };
                
                for &start in &side.pieces {
                    let piece = self.board[start as usize].get_type();
    
                    if pins.iter().find(|&piece| piece.0 == start).is_none() {
                        match piece {
                            QUEEN | ROOK | BISHOP => self.generate_sliding_defenses(&mut moves, start, piece, &defend_squares),
                            KNIGHT => self.generate_knight_defenses(&mut moves, start, &defend_squares),
                            PAWN if self.colour_turn == WHITE => self.generate_white_pawn_moves(&mut moves, start, en_passant_able, is_pawn_check, Some(&defend_squares)),
                            _ => self.generate_black_pawn_moves(&mut moves, start, en_passant_able, is_pawn_check, Some(&defend_squares)),
                        }
                    }
                }
            }
        }
        self.generate_king_moves(&mut moves, side.king_square, opponent_side);
        moves
    }

    fn get_pins_and_slide_attack(&self, square: u8, colour: Colour, side: &SideState, opponent_side: &SideState) -> (Vec<(u8, Vec<u8>)>, Option<Vec<u8>>, bool) {
        let mut pins = Vec::new();
        let mut attack = None;
        let mut en_passant_able = true;
        for dir_index in 0..8 {
            let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[square as usize][dir_index]};
            let mut zone = Vec::new();
            'outer: for n in 0..dist_edge {
                let pinned_square = (square as i8 + DIRECTION_OFFSETS[dir_index] * (n+1) as i8) as u8;
                let pinned_piece = self.board[pinned_square as usize];
                zone.push(pinned_square);

                if pinned_piece == NONE {
                    //
                }
                else if pinned_piece.is_colour(colour) {
                    /*if opponent_side.slide_control[pinned_square as usize] == 0 {
                        continue;
                    } else */{
                        for n in (n+1)..dist_edge {
                            let target_square = (square as i8 + DIRECTION_OFFSETS[dir_index] * (n+1) as i8) as u8;
                            let target_piece = self.board[target_square as usize];
                            
                            zone.push(target_square);
                            if target_piece == NONE {
                                if opponent_side.slide_control[target_square as usize] == 0 {
                                    //break 'outer;
                                }
                            }
                            else if target_piece.is_colour(colour.opponent()) {
                                let target_type = target_piece.get_type();
                                if target_type.is_sliding()
                                && target_type.get_sliding_indices().contains(&dir_index) {
                                    pins.push((pinned_square, zone));
                                }
                                break 'outer;
                            }
                            else {
                                break 'outer;
                            }
                        }
                    }
                }
                else {
                    let pinned_type = pinned_piece.get_type();
                    if pinned_type.is_sliding()
                    && pinned_type.get_sliding_indices().contains(&dir_index) {
                        zone.push(pinned_square);
                        attack = Some(zone);
                    }
                    break;
                }
            }
        }
        if self.en_passant != u8::MAX {
            let (rank, pawn_heading) = if self.colour_turn == WHITE {(4,8)} else {(3,-8)};
            if square / 8 == rank {
                let dir_index = if square % 8 > self.en_passant % 8 {3} else {2};
                let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[square as usize][dir_index]};
                
                let mut pinned_pawn = u8::MAX;
                let mut pushser = false;

                for n in 0..dist_edge {
                    let target_square = (square as i8 + DIRECTION_OFFSETS[dir_index] * (n+1) as i8) as u8;
                    let target_piece = self.board[target_square as usize];
                    let target_type = target_piece.get_type();

                    if target_piece == NONE {
                        //
                    } else if target_type == PAWN {
                        if target_piece.is_colour(colour) {
                            if pinned_pawn == u8::MAX {
                                pinned_pawn = target_square;
                            } else {
                                break;
                            }
                        } else {
                            if target_square as i8 + pawn_heading as i8 == self.en_passant as i8 {
                                pushser = true;
                            } else {
                                break;
                            }
                        }
                    } else {
                        if target_piece.is_colour(colour) {break}
                        if !pushser || pinned_pawn == u8::MAX {break}
                        
                        if target_type.is_sliding()
                        && target_type.get_sliding_indices().contains(&dir_index) {
                            let mut defense = (pinned_pawn as i8 + pawn_heading - DIRECTION_OFFSETS[dir_index]) as u8;
                            if defense == self.en_passant {
                                defense = (pinned_pawn as i8 + pawn_heading + DIRECTION_OFFSETS[dir_index]) as u8;
                            }
                            en_passant_able = false;
                        }
                        break;
                    }
                }
            }
        }
        (pins, attack, en_passant_able)
    }

    fn generate_sliding_moves(&self, moves: &mut Vec<Moves>, start: u8, piece: PieceType) {
        for dir_index in piece.get_sliding_indices() {
            let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[start as usize][dir_index]};
            for n in 0..dist_edge {
                let target = (start as i8 + DIRECTION_OFFSETS[dir_index] * (n + 1) as i8) as u8;
                let target_piece = self.board[target as usize];

                if target_piece.is_colour(self.colour_turn) {break}

                moves.push(Moves::Move { start, target });

                if target_piece != NONE {break}
            }
        }
    }
    fn generate_sliding_defenses(&self, moves: &mut Vec<Moves>, start: u8, piece: PieceType, defend_squares: &Vec<u8>) {
        for dir_index in piece.get_sliding_indices() {
            let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[start as usize][dir_index]};
            for n in 0..dist_edge {
                let target = (start as i8 + DIRECTION_OFFSETS[dir_index] * (n + 1) as i8) as u8;
                let target_piece = self.board[target as usize];

                if target_piece.is_colour(self.colour_turn) {break}

                if defend_squares.contains(&target) {
                    moves.push(Moves::Move { start, target });
                }

                if target_piece != NONE {break}
            }
        }
    }

    fn generate_white_pawn_moves(&self, moves: &mut Vec<Moves>, start: u8, en_passant_able: bool, is_pawn_check: bool, defend_squares: Option<&Vec<u8>>) {
        if start / 8 == 6 {
            for &target in unsafe {&WHITE_PAWN_CONTROL[start as usize]} {
                if self.board[target as usize].is_colour(BLACK) {
                    if defend_squares.map_or(true, |squares| squares.contains(&target)) {
                        for piece in [QUEEN, KNIGHT, ROOK, BISHOP] {
                            moves.push(Moves::Promotion { start, target, piece });
                        }
                    }
                }
            }
            let target = start + 8;
            if self.board[target as usize] == NONE {
                if defend_squares.map_or(true, |squares| squares.contains(&target)) {
                    for piece in [QUEEN, KNIGHT, ROOK, BISHOP] {
                        moves.push(Moves::Promotion { start, target, piece });
                    }
                }
            }
        }
        else {
            for &target in unsafe {&WHITE_PAWN_CONTROL[start as usize]} {
                if self.board[target as usize].is_colour(BLACK) {
                    if defend_squares.map_or(true, |squares| squares.contains(&target)) {
                        moves.push(Moves::Move { start, target });
                    }
                }
                else if target == self.en_passant {
                    if en_passant_able
                    && (is_pawn_check
                    || defend_squares.map_or(true, |squares| squares.contains(&target))) {
                        moves.push(Moves::EnPassant { start, target });
                    }
                }
            }
            let mut target = start + 8;
            if self.board[target as usize] == NONE {
                if defend_squares.map_or(true, |squares| squares.contains(&target)) {
                    moves.push(Moves::Move { start, target });
                }
                if start / 8 == 1 {
                    target += 8;
                    if self.board[target as usize] == NONE {
                        if defend_squares.map_or(true, |squares| squares.contains(&target)) {
                            moves.push(Moves::DoublePush { start, target });
                        }
                    }
                }
            }
        }
    }
    fn generate_black_pawn_moves(&self, moves: &mut Vec<Moves>, start: u8, en_passant_able: bool, is_pawn_check: bool, defend_squares: Option<&Vec<u8>>) {
        if start / 8 == 1 {
            for &target in unsafe {&BLACK_PAWN_CONTROL[start as usize]} {
                if self.board[target as usize].is_colour(WHITE) {
                    if defend_squares.map_or(true, |squares| squares.contains(&target)) {
                        for piece in [QUEEN, KNIGHT, ROOK, BISHOP] {
                            moves.push(Moves::Promotion { start, target, piece });
                        }
                    }
                }
            }
            let target = start - 8;
            if self.board[target as usize] == NONE {
                if defend_squares.map_or(true, |squares| squares.contains(&target)) {
                    for piece in [QUEEN, KNIGHT, ROOK, BISHOP] {
                        moves.push(Moves::Promotion { start, target, piece });
                    }
                }
            }
        }
        else {
            for &target in unsafe {&BLACK_PAWN_CONTROL[start as usize]} {
                if self.board[target as usize].is_colour(WHITE) {
                    if defend_squares.map_or(true, |squares| squares.contains(&target)) {
                        moves.push(Moves::Move { start, target });
                    }
                }
                else if target == self.en_passant {
                    if en_passant_able
                    && (is_pawn_check
                    || defend_squares.map_or(true, |squares| squares.contains(&target))) {
                        moves.push(Moves::EnPassant { start, target });
                    }
                }
            }
            let mut target = start - 8;
            if self.board[target as usize] == NONE {
                if defend_squares.map_or(true, |squares| squares.contains(&target)) {
                    moves.push(Moves::Move { start, target });
                }
                if start / 8 == 6 {
                    target -= 8;
                    if self.board[target as usize] == NONE {
                        if defend_squares.map_or(true, |squares| squares.contains(&target)) {
                            moves.push(Moves::DoublePush { start, target });
                        }
                    }
                }
            }
        }
    }
    fn find_white_pawn_check(&self, moves: &mut Vec<Moves>, start: u8) -> u8 {
        for &target in unsafe {&BLACK_PAWN_CONTROL[start as usize]} {
            if self.board[target as usize] == Piece::new(PAWN, WHITE) {
                return target;
            }
        }
        return u8::MAX;
    }
    fn find_black_pawn_check(&self, moves: &mut Vec<Moves>, start: u8) -> u8 {
        for &target in unsafe {&WHITE_PAWN_CONTROL[start as usize]} {
            if self.board[target as usize] == Piece::new(PAWN, BLACK) {
                return target;
            }
        }
        return u8::MAX;
    }

    fn generate_knight_moves(&self, moves: &mut Vec<Moves>, start: u8) {
        for &target in unsafe {&KNIGHT_CONTROL[start as usize]} {
            if !self.board[target as usize].is_colour(self.colour_turn) {
                moves.push(Moves::Move { start, target });
            }
        }
    }
    fn generate_knight_defenses(&self, moves: &mut Vec<Moves>, start: u8, defend_squares: &Vec<u8>) {
        for &target in unsafe {&KNIGHT_CONTROL[start as usize]} {
            if defend_squares.contains(&target) {
                moves.push(Moves::Move { start, target });
            }
        }
    }
    fn find_knight_check(&self, moves: &mut Vec<Moves>, start: u8) -> u8 {
        for &target in unsafe {&KNIGHT_CONTROL[start as usize]} {
            if self.board[target as usize] == Piece::new(KNIGHT, self.colour_turn.opponent()) {
                return target;
            }
        }
        return u8::MAX;
    }
    
    fn generate_king_moves(&self, moves: &mut Vec<Moves>, start: u8, opponent_side: &SideState) {
        for &target in unsafe {&KING_CONTROL[start as usize]} {
            if !self.board[target as usize].is_colour(self.colour_turn)
            && opponent_side.control[target as usize] == 0 {
                moves.push(Moves::Move { start, target })
            }
        }
        if self.colour_turn == WHITE {
            if self.castling & CASTLE_WHITE_KING != 0 {
                if self.board[5] == NONE
                && self.board[6] == NONE
                && opponent_side.control[4] == 0
                && opponent_side.control[5] == 0
                && opponent_side.control[6] == 0 {
                    moves.push(Moves::KingCastling);
                }
            }
            if self.castling & CASTLE_WHITE_QUEEN != 0 {
                if self.board[1] == NONE
                && self.board[2] == NONE
                && self.board[3] == NONE
                && opponent_side.control[2] == 0
                && opponent_side.control[3] == 0
                && opponent_side.control[4] == 0 {
                    moves.push(Moves::QueenCastling);
                }
            }
        }
        else {
            if self.castling & CASTLE_BLACK_KING != 0 {
                if self.board[61] == NONE
                && self.board[62] == NONE
                && opponent_side.control[60] == 0
                && opponent_side.control[61] == 0
                && opponent_side.control[62] == 0 {
                    moves.push(Moves::KingCastling);
                }
            }
            if self.castling & CASTLE_BLACK_QUEEN != 0 {
                if self.board[57] == NONE
                && self.board[58] == NONE
                && self.board[59] == NONE
                && opponent_side.control[58] == 0
                && opponent_side.control[59] == 0
                && opponent_side.control[60] == 0 {
                    moves.push(Moves::QueenCastling);
                }
            }
        }
    }
}