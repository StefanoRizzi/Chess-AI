use std::{cmp::min, sync::Mutex, str::FromStr};

use super::*;

static PRECOMPUTED: std::sync::Mutex<bool> = Mutex::new(false);

pub const DIRECTION_OFFSETS: [i8;8] = [8,-8,1,-1,7,-7,9,-9];
pub const DIRECTION_OPPOST_INDEX: [usize;8] = [1,0,3,2,5,4,7,6];
pub static mut NUM_SQUARES_TO_EDGES: [[u8;8]; 64] = [[0;8]; 64];

const NEW_VEC: Vec<Square> = Vec::new();
pub static mut KING_ATTACKS: [Vec<Square>;64] = [NEW_VEC;64];
pub static mut KNIGHT_ATTACKS: [Vec<Square>;64] = [NEW_VEC;64];
pub static mut PAWN_ATTACKS: [[Vec<Square>;64]; 2] = [[NEW_VEC;64], [NEW_VEC;64]];

pub fn piece_attacks(piece_type: PieceType, colour: Colour, square: Square) -> &'static mut Vec<Square> {
    &mut (match piece_type {
        KING => unsafe {&mut KING_ATTACKS},
        KNIGHT => unsafe {&mut KNIGHT_ATTACKS},
        PAWN => unsafe {&mut PAWN_ATTACKS[colour.colour_index()]},
        _ => unreachable!(),
    })[square as usize]
}
pub fn precompute() { // mutex because rust is bugged on tests
    let mut precomputed = PRECOMPUTED.lock().unwrap();
    if *precomputed {return}
    *precomputed = true;
    
    let path = PathBuf::from_str(&std::env::var("HOME").unwrap()).unwrap().join(".chess-AI");
    let _ = std::fs::create_dir(&path);
    *ROOT_PATH.lock().unwrap() = Some(path);

    clear_log();
    precomputed_move_data();
    precomputed_king_attacks();
    precomputed_knight_attacks();
    precomputed_pawn_attacks();
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
fn precomputed_king_attacks() {
    for square in 0..64 {
        for dir_index in 0..8 {
            let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[square as usize][dir_index]};
            if dist_edge >= 1 {
                let target = square + DIRECTION_OFFSETS[dir_index];
                unsafe {KING_ATTACKS[square as usize].push(target)};
            }
        }
    }
}
fn precomputed_knight_attacks() {
    for square in 0..64 {
        for jump in [-17, -15, -10, -6, 6, 10, 15, 17] {
            let target = square + jump;
            if target >= 64 || target < 0 {continue}
            if square % 8 - target % 8 > 2 {continue}
            if square % 8 - target % 8 < -2 {continue}
            unsafe {KNIGHT_ATTACKS[square as usize].push(target)};
        }
    }
}
fn precomputed_pawn_attacks() {
    for square in 0..64 {
        for (colour_index, attacks) in [[4, 6], [5, 7]].iter().enumerate() {
            for &dir_index in attacks {
                let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[square as usize][dir_index]};
                if dist_edge >= 1 {
                    let target = square + DIRECTION_OFFSETS[dir_index];
                    unsafe {PAWN_ATTACKS[colour_index][square as usize].push(target)};
                }
            }
        }
    }
}

impl Chess {
    pub fn generate_legal_moves(&self) -> Vec<Move> {

        let colour = self.colour_to_move();
        let colour_index = self.colour_index();
        let opponent_index = self.opponent_index();
        let king = self.side[colour_index].king;

        let (mut pins, attack) = self.get_pins_and_slide_attack(king, colour);
        let is_en_passant_pinned = self.is_en_passant_pinned();

        let mut moves = Vec::new();
        
        self.generate_king_moves(&mut moves);

        if !self.is_king_in_check() {
            let sliding_pieces = self.side[colour_index].queens.iter();
            let piece_type = QUEEN;
            for &start in sliding_pieces {
                if let Some(pin_index) = pins.iter().position(|pin| pin.0 == start) {
                    let defend_squares = pins.swap_remove(pin_index).1;
                    self.generate_sliding_defenses(&mut moves, start, piece_type, &defend_squares);
                } else {
                    self.generate_sliding_moves(&mut moves, start, piece_type);
                }
            }
            let sliding_pieces = self.side[colour_index].rooks.iter();
            let piece_type = ROOK;
            for &start in sliding_pieces {
                if let Some(pin_index) = pins.iter().position(|pin| pin.0 == start) {
                    let defend_squares = pins.swap_remove(pin_index).1;
                    self.generate_sliding_defenses(&mut moves, start, piece_type, &defend_squares);
                } else {
                    self.generate_sliding_moves(&mut moves, start, piece_type);
                }
            }
            let sliding_pieces = self.side[colour_index].bishops.iter();
            let piece_type = BISHOP;
            for &start in sliding_pieces {
                if let Some(pin_index) = pins.iter().position(|pin| pin.0 == start) {
                    let defend_squares = pins.swap_remove(pin_index).1;
                    self.generate_sliding_defenses(&mut moves, start, piece_type, &defend_squares);
                } else {
                    self.generate_sliding_moves(&mut moves, start, piece_type);
                }
            }
            let sliding_pieces = self.side[colour_index].knights.iter();
            for &start in sliding_pieces {
                if let Some(pin_index) = pins.iter().position(|pin| pin.0 == start) {
                    let defend_squares = pins.swap_remove(pin_index).1;
                    self.generate_knight_defenses(&mut moves, start, &defend_squares);
                } else {
                    self.generate_knight_moves(&mut moves, start);
                }
            }
            let sliding_pieces = self.side[colour_index].pawns.iter();
            for &start in sliding_pieces {
                if let Some(pin_index) = pins.iter().position(|pin| pin.0 == start) {
                    let defend_squares = pins.swap_remove(pin_index).1;
                    self.generate_pawn_defenses(&mut moves, start, false, &defend_squares);
                } else {
                    self.generate_pawn_moves(&mut moves, start, is_en_passant_pinned);
                }
            }
        }
        else if self.get_king_treats() == 1 {
            let defend_squares = attack.unwrap_or_else(|| Vec::from([
                if self.side[opponent_index].piece_attacks[KNIGHT.piece_index()][king as usize] == 1
                {self.find_knight_check()}
                else
                {self.find_pawn_check()}
                ]));
            let is_en_passant_defense = self.side[opponent_index].piece_attacks[PAWN.piece_index()][king as usize] == 1;
            
            let sliding_pieces = self.side[colour_index].queens.iter();
            let piece_type = QUEEN;
            for &start in sliding_pieces {
                if !pins.iter().any(|pin| pin.0 == start) {
                    self.generate_sliding_defenses(&mut moves, start, piece_type, &defend_squares);
                }
            }
            let sliding_pieces = self.side[colour_index].rooks.iter();
            let piece_type = ROOK;
            for &start in sliding_pieces {
                if !pins.iter().any(|pin| pin.0 == start) {
                    self.generate_sliding_defenses(&mut moves, start, piece_type, &defend_squares);
                }
            }
            let sliding_pieces = self.side[colour_index].bishops.iter();
            let piece_type = BISHOP;
            for &start in sliding_pieces {
                if !pins.iter().any(|pin| pin.0 == start) {
                    self.generate_sliding_defenses(&mut moves, start, piece_type, &defend_squares);
                }
            }
            let sliding_pieces = self.side[colour_index].knights.iter();
            for &start in sliding_pieces {
                if !pins.iter().any(|pin| pin.0 == start) {
                    self.generate_knight_defenses(&mut moves, start, &defend_squares);
                }
            }
            let sliding_pieces = self.side[colour_index].pawns.iter();
            for &start in sliding_pieces {
                if !pins.iter().any(|pin| pin.0 == start) {
                    self.generate_pawn_defenses(&mut moves, start, is_en_passant_defense, &defend_squares);
                }
            }
        }
        moves
    }

    fn get_pins_and_slide_attack(&self, square: Square, colour: Colour) -> (Vec<(Square, Vec<Square>)>, Option<Vec<Square>>) {
        let mut pins = Vec::new();
        let mut attack = None;

        for dir_index in 0..8 {
            let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[square as usize][dir_index]};
            let mut defend_squares = Vec::new();

            for n in 0..dist_edge {
                let pinned_square = square + DIRECTION_OFFSETS[dir_index] * (n+1) as i8;
                let pinned_piece = self.board[pinned_square as usize];
                
                defend_squares.push(pinned_square);
                if pinned_piece == NONE {continue}

                if pinned_piece.is_colour(colour) {

                    for n2 in (n+1)..dist_edge {
                        let target_square = square + DIRECTION_OFFSETS[dir_index] * (n2+1) as i8;
                        let target_piece = self.board[target_square as usize];
                        
                        defend_squares.push(target_square);
                        if target_piece == NONE {continue}

                        if target_piece.is_colour(colour.opponent()) {
                            if target_piece.get_type().is_sliding()
                            && target_piece.get_type().get_sliding_indices().contains(&dir_index) {
                                pins.push((pinned_square, defend_squares));
                            }
                        }
                        break;
                    }

                }
                else {
                    if pinned_piece.get_type().is_sliding()
                    && pinned_piece.get_type().get_sliding_indices().contains(&dir_index) {
                        attack = Some(defend_squares);
                    }
                }
                break;
            }
        }
        (pins, attack)
    }
    
    pub fn is_en_passant_pinned(&self) -> bool {
        if self.en_passant == -1 {return false}
        
        let colour = self.colour_to_move();
        let king = self.side[self.colour_index()].king;
        let rank = if self.is_white_to_move {4} else {3};
        
        if king / 8 != rank {return false}
        
        let mut pinned_pawn = -1;
        let dir_index = if king % 8 > self.en_passant % 8 {3} else {2};
        
        let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[king as usize][dir_index]};
        
        for n in 0..dist_edge {
            let target_square = king + DIRECTION_OFFSETS[dir_index] * (n+1) as i8;
            let target_piece = self.board[target_square as usize];

            if target_piece == NONE {continue}
            
            if target_piece.get_type() == PAWN {

                if target_piece.is_colour(colour) {
                    if pinned_pawn == -1 {
                        pinned_pawn = target_square;
                        continue;
                    }
                } else {
                    if target_square % 8 == self.en_passant % 8 {
                        continue;
                    }
                }

            }
            else if !target_piece.is_colour(colour)
            && pinned_pawn != -1 {
                if target_piece.get_type().is_sliding()
                && target_piece.get_type().get_sliding_indices().contains(&dir_index) {
                    return true;
                }
            }
            break;
        }
        return false;
    }

    fn generate_sliding_moves(&self, moves: &mut Vec<Move>, start: Square, piece: PieceType) {
        for dir_index in piece.get_sliding_indices() {
            let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[start as usize][dir_index]};

            for n in 0..dist_edge {
                let target = start + DIRECTION_OFFSETS[dir_index] * (n + 1) as i8;
                let target_piece = self.board[target as usize];

                if target_piece.is_colour(self.colour_to_move()) {break}

                moves.push(Move::new(start, target, NO_FLAG));

                if target_piece != NONE {break}
            }
        }
    }
    fn generate_sliding_defenses(&self, moves: &mut Vec<Move>, start: Square, piece: PieceType, defend_squares: &Vec<Square>) {
        for dir_index in piece.get_sliding_indices() {
            let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[start as usize][dir_index]};
            for n in 0..dist_edge {
                let target = start + DIRECTION_OFFSETS[dir_index] * (n + 1) as i8;
                let target_piece = self.board[target as usize];

                if target_piece.is_colour(self.colour_to_move()) {break}

                if defend_squares.contains(&target) {
                    moves.push(Move::new(start, target, NO_FLAG));
                }

                if target_piece != NONE {break}
            }
        }
    }

    fn generate_pawn_moves(&self, moves: &mut Vec<Move>, start: Square, is_en_passant_pinned: bool) {
        let opponent_color = self.colour_to_move().opponent();

        let (double_push_rank, promotion_rank, pawn_heading) = if self.is_white_to_move {(1,6,8)} else {(6,1,-8)};
        let pawn_attacks = unsafe {&PAWN_ATTACKS[self.colour_index()]};

        if start / 8 == promotion_rank {

            for &target in &pawn_attacks[start as usize] {
                if self.board[target as usize].is_colour(opponent_color) {
                    for promotion_flag in [PROMOTE_TO_QUEEN_FLAG, PROMOTE_TO_KNIGHT_FLAG, PROMOTE_TO_ROOK_FLAG, PROMOTE_TO_BISHOP_FLAG] {
                        moves.push(Move::new(start, target, promotion_flag));
                    }
                }
            }
            let target = start + pawn_heading;

            if self.board[target as usize] == NONE {
                for promotion_flag in [PROMOTE_TO_QUEEN_FLAG, PROMOTE_TO_KNIGHT_FLAG, PROMOTE_TO_ROOK_FLAG, PROMOTE_TO_BISHOP_FLAG] {
                    moves.push(Move::new(start, target, promotion_flag));
                }
            }

        }
        else {

            for &target in &pawn_attacks[start as usize] {
                if self.board[target as usize].is_colour(opponent_color) {
                    moves.push(Move::new(start, target, NO_FLAG));
                }
                else if target == self.en_passant && !is_en_passant_pinned {
                    moves.push(Move::new(start, target, EN_PASSANT_FLAG));
                }
            }
            let target = start + pawn_heading;

            if self.board[target as usize] == NONE {
                moves.push(Move::new(start, target, NO_FLAG));
                
                if start / 8 == double_push_rank {
                    let target = target + pawn_heading;

                    if self.board[target as usize] == NONE {
                        moves.push(Move::new(start, target, DOUBLE_PUSH_FLAG));
                    }
                }
            }
        }
    }
    fn generate_pawn_defenses(&self, moves: &mut Vec<Move>, start: Square, is_en_passant_defense: bool, defend_squares: &Vec<Square>) {
        let opponent_color = self.colour_to_move().opponent();

        let (double_push_rank, promotion_rank, pawn_heading) = if self.is_white_to_move {(1,6,8)} else {(6,1,-8)};
        let pawn_attacks = unsafe {&PAWN_ATTACKS[self.colour_index()]};

        if start / 8 == promotion_rank {

            for &target in &pawn_attacks[start as usize] {
                if self.board[target as usize].is_colour(opponent_color) {
                    if defend_squares.contains(&target) {
                        for promotion_flag in [PROMOTE_TO_QUEEN_FLAG, PROMOTE_TO_KNIGHT_FLAG, PROMOTE_TO_ROOK_FLAG, PROMOTE_TO_BISHOP_FLAG] {
                            moves.push(Move::new(start, target, promotion_flag));
                        }
                    }
                }
            }
            let target = start + pawn_heading;

            if self.board[target as usize] == NONE {
                if defend_squares.contains(&target) {
                    for promotion_flag in [PROMOTE_TO_QUEEN_FLAG, PROMOTE_TO_KNIGHT_FLAG, PROMOTE_TO_ROOK_FLAG, PROMOTE_TO_BISHOP_FLAG] {
                        moves.push(Move::new(start, target, promotion_flag));
                    }
                }
            }

        }
        else {

            for &target in &pawn_attacks[start as usize] {
                if defend_squares.contains(&target) {
                    if self.board[target as usize].is_colour(opponent_color) {
                        moves.push(Move::new(start, target, NO_FLAG));
                    }
                }
                if target == self.en_passant && is_en_passant_defense {
                    moves.push(Move::new(start, target, EN_PASSANT_FLAG));
                }
            }
            let target = start + pawn_heading;

            if self.board[target as usize] == NONE {

                if defend_squares.contains(&target) {
                    moves.push(Move::new(start, target, NO_FLAG));
                }
                
                if start / 8 == double_push_rank {
                    let target = target + pawn_heading;

                    if self.board[target as usize] == NONE {
                        if defend_squares.contains(&target) {
                            moves.push(Move::new(start, target, DOUBLE_PUSH_FLAG));
                        }
                    }
                }
            }
        }
    }
    
    fn find_pawn_check(&self) -> Square {
        let pawn_attacks = unsafe {&PAWN_ATTACKS[self.colour_index()]};
        for &target in &pawn_attacks[self.get_king_square() as usize] {
            if self.board[target as usize] == Piece::new(PAWN, self.colour_to_move().opponent()) {
                return target;
            }
        }
        self.display();
        self.display_attacks(BLACK);
        Chess::display_attacks_pieces(&self.side[self.opponent_index()].piece_attacks[KNIGHT.piece_index()]);
        Chess::display_attacks_pieces(&self.side[self.opponent_index()].piece_attacks[PAWN.piece_index()]);
        Chess::display_attacks_pieces(&self.side[self.opponent_index()].piece_attacks[ROOK.piece_index()]);
        Chess::display_attacks_pieces(&self.side[self.opponent_index()].piece_attacks[BISHOP.piece_index()]);
        Chess::display_attacks_pieces(&self.side[self.opponent_index()].piece_attacks[QUEEN.piece_index()]);
        unreachable!()
    }

    fn generate_knight_moves(&self, moves: &mut Vec<Move>, start: Square) {
        for &target in unsafe {&KNIGHT_ATTACKS[start as usize]} {
            if !self.board[target as usize].is_colour(self.colour_to_move()) {
                moves.push(Move::new(start, target, NO_FLAG));
            }
        }
    }
    fn generate_knight_defenses(&self, moves: &mut Vec<Move>, start: Square, defend_squares: &Vec<Square>) {
        for &target in unsafe {&KNIGHT_ATTACKS[start as usize]} {
            if defend_squares.contains(&target) {
                moves.push(Move::new(start, target, NO_FLAG));
            }
        }
    }
    fn find_knight_check(&self) -> Square {
        for &target in unsafe {&KNIGHT_ATTACKS[self.get_king_square() as usize]} {
            if self.board[target as usize] == Piece::new(KNIGHT, self.colour_to_move().opponent()) {
                return target;
            }
        }
        self.display();
        self.display_attacks(BLACK);
        unreachable!()
    }
    
    fn generate_king_moves(&self, moves: &mut Vec<Move>) {
        let colour = self.colour_to_move();
        let opponent_attacks = &self.side[self.opponent_index()].attacks;
        let start = self.side[self.colour_index()].king;

        for &target in unsafe {&KING_ATTACKS[start as usize]} {
            if !self.board[target as usize].is_colour(colour)
            && opponent_attacks[target as usize] == 0 {
                moves.push(Move::new(start, target, NO_FLAG))
            }
        }
        if self.is_white_to_move {
            if self.castling & CASTLE_WHITE_KING != 0 {
                if self.board[5] == NONE
                && self.board[6] == NONE
                && opponent_attacks[4] == 0
                && opponent_attacks[5] == 0
                && opponent_attacks[6] == 0 {
                    moves.push(Move::new(4, 6, CASTLE_FLAG));
                }
            }
            if self.castling & CASTLE_WHITE_QUEEN != 0 {
                if self.board[1] == NONE
                && self.board[2] == NONE
                && self.board[3] == NONE
                && opponent_attacks[2] == 0
                && opponent_attacks[3] == 0
                && opponent_attacks[4] == 0 {
                    moves.push(Move::new(4, 2, CASTLE_FLAG));
                }
            }
        }
        else {
            if self.castling & CASTLE_BLACK_KING != 0 {
                if self.board[61] == NONE
                && self.board[62] == NONE
                && opponent_attacks[60] == 0
                && opponent_attacks[61] == 0
                && opponent_attacks[62] == 0 {
                    moves.push(Move::new(60, 62, CASTLE_FLAG));
                }
            }
            if self.castling & CASTLE_BLACK_QUEEN != 0 {
                if self.board[57] == NONE
                && self.board[58] == NONE
                && self.board[59] == NONE
                && opponent_attacks[58] == 0
                && opponent_attacks[59] == 0
                && opponent_attacks[60] == 0 {
                    moves.push(Move::new(60, 58, CASTLE_FLAG));
                }
            }
        }
    }
}