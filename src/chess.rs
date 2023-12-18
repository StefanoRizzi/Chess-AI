// SETTINGS
pub static mut DISPLAY: bool = true;

// max turn 30 * 50 = 1500
const MAX_DEPTH: usize = 3000;
const MAX_MOVES: usize = 218;
const START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

mod fen;
mod display;
pub mod piece;
pub mod legal_moves;

pub use piece::*;

use castle::*;
use legal_moves::*;

pub mod castle {
    pub const CASTLE_NONE: u8 = 0;
    pub const CASTLE_WHITE_QUEEN: u8 = 1;
    pub const CASTLE_WHITE_KING: u8 = 2;
    pub const CASTLE_BLACK_QUEEN: u8 = 4;
    pub const CASTLE_BLACK_KING: u8 = 8;
}

pub type Square = u8;
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Moves {
    Move{start: Square, target: Square},
    QueenCastling,
    KingCastling,
    Promotion{start: Square, target: Square, promotion_type: PieceType},
    DoublePush{start: Square, target: Square},
    EnPassant{start: Square, target: Square},
}

#[derive(Debug, Clone, PartialEq)]
pub struct SideState {
    pub king: Square,
    pub pawns: Vec<Square>,
    pub knights: Vec<Square>,
    pub bishops: Vec<Square>,
    pub rooks: Vec<Square>,
    pub queens: Vec<Square>,
    pub attacks: [i8; 64],
    pub piece_attacks: [[i8; 64]; 6],
}
impl Default for SideState {
    fn default() -> Self {
        let mut side = Self { king: Default::default(), pawns: Default::default(), knights: Default::default(), bishops: Default::default(), rooks: Default::default(), queens: Default::default(), attacks: [0; 64], piece_attacks: [[0; 64]; 6] };
        side.pawns.reserve_exact(8);
        side.knights.reserve_exact(10);
        side.bishops.reserve_exact(10);
        side.rooks.reserve_exact(10);
        side.queens.reserve_exact(10);
        return side
    }
}
impl SideState {
    pub fn pieces(&mut self, piece_type: PieceType) -> &mut Vec<Square> {
        match piece_type {
            KING => unreachable!(),
            PAWN => &mut self.pawns,
            KNIGHT => &mut self.knights,
            BISHOP => &mut self.bishops,
            ROOK => &mut self.rooks,
            QUEEN => &mut self.queens,
            _ => unreachable!(),
        }
    }
    fn new_piece(&mut self, piece_type: PieceType, square: Square) {
        match piece_type {
            KING => self.king = square,
            _ => self.pieces(piece_type).push(square),
        }
    }
    
    fn remove_piece(&mut self, piece_type: PieceType, square: Square) {
        let pieces = self.pieces(piece_type);
        pieces.swap_remove(pieces.iter().position(|&p|p == square).unwrap());
    }
    
    fn move_piece(&mut self, piece_type: PieceType, start: Square, target: Square) {
        match piece_type {
            KING => self.king = target,
            _ => {
                let pieces = self.pieces(piece_type);
                let index = pieces.iter().position(|&p|p == start).unwrap();
                pieces[index] = target;
            }
        }
    }
}
impl Chess {
    fn spread_attack_direcion(&mut self, piece_type: PieceType, colour: Colour, start: Square, dir_index: usize, value: i8) {
        let dir = DIRECTION_OFFSETS[dir_index];
        let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[start as usize][dir_index]};
        let piece_index = piece_type.piece_index();
        let colour_index = colour.colour_index();
        let opponent_king = self.side[colour.opponent().colour_index()].king;
        for n in 0..dist_edge {
            let target = start as i8 + ((n+1) as i8*dir);
            let piece = self.board[target as usize];
            self.side[colour_index].piece_attacks[piece_index][target as usize] += value;
            self.side[colour_index].attacks[target as usize] += value;
            
            if piece == NONE || target as Square == opponent_king {continue}

            break;
        }
    }
    fn update_attack_pieces(&mut self, piece_type: PieceType, colour: Colour, square: Square, value: i8) {
        let mut white_control = self.side[WHITE.colour_index()].attacks[square as usize] > 0;
        let mut black_control = self.side[BLACK.colour_index()].attacks[square as usize] > 0;
        white_control &= Piece::new(piece_type, colour) != Piece::new(KING, BLACK);
        black_control &= Piece::new(piece_type, colour) != Piece::new(KING, WHITE);

        if !(white_control || black_control) {return}

        for dir_index in 0..8 {

            let dir = DIRECTION_OFFSETS[dir_index];
            let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[square as usize][dir_index]};
            
            for n in 0..dist_edge {
                let target = square as i8 + ((n+1) as i8*dir);
                let target_piece = self.board[target as usize];
                if target_piece == NONE {continue}
                
                let is_target_white = target_piece.is_colour(WHITE);
                if (white_control && is_target_white)
                || (black_control && !is_target_white) {
                    if target_piece.get_type().is_sliding()
                    && target_piece.get_type().get_sliding_indices().contains(&dir_index) {
                        self.spread_attack_direcion(target_piece.get_type(), Colour::new(is_target_white), square, DIRECTION_OPPOST_INDEX[dir_index], value);
                    }
                }
                break;
            }
        }
    }
    fn piece_attack(&mut self, piece_type: PieceType, colour: Colour, square: Square, value: i8) {
        if piece_type.is_sliding() {
            for dir_index in piece_type.get_sliding_indices() {
                self.spread_attack_direcion(piece_type, colour, square, dir_index, value);
            }
        }
        else {
            let colour_index = colour.colour_index();
            let piece_index = piece_type.piece_index();
            
            for &mut target in piece_attacks(piece_type, colour, square) {
                self.side[colour_index].piece_attacks[piece_index][target as usize] += value;
                self.side[colour_index].attacks[target as usize] += value;
            }
        }
    }
    pub fn put_attack_and_update(&mut self, piece_type: PieceType, colour: Colour, square: Square) {
        self.piece_attack(piece_type, colour, square, 1);
        self.update_attack_pieces(piece_type, colour, square, -1);
    }
    pub fn remove_attack_and_update(&mut self, piece_type: PieceType, colour: Colour, square: Square) {
        self.piece_attack(piece_type, colour, square, -1);
        self.update_attack_pieces(piece_type, colour, square, 1);
    }
}

/*
/========================\ /========================\
| r :n: b :q: k :b: n :r:| |   :::   :::   :::   :::|
|:p: p :p: p :p: p :p: p | |:::   :::   :::   :::   |
|   :::   :::   :::   :::| |   :::   :::   :::   :::|
|:::   :::   :::   :::   | |:::   :::   :::   :::   |
|   :::   :::   :::   :::| |   :::   :::   :::   :::|
|:::   :::   :::   :::   | |:::   :::   :::   :::   |
| P :P: P :P: P :P: P :P:| |   :::   :::   :::   :::|
|:R: N :B: Q :K: B :N: R | |:::   :::   :::   :::   |
\========================/ \========================/
*/
#[derive(Debug, Clone, PartialEq)]
pub struct Chess {
    pub board: [Piece; 64],
    pub en_passant: Square,
    pub castling: u8,
    pub half_move: u16,
    pub full_turn: u16,
    pub is_white_to_move: bool,
    pub side: [SideState; 2],
    // eaten piece | en passant | castlilg | half move clock
    pub irreversable_state: Vec<(PieceType, Square, u8, u16)>,
    pub moves_history: Vec<Moves>,
}


impl Chess {
    pub fn new() -> Chess {Chess::build(START_POSITION)}
    pub fn colour_to_move(&self) -> Colour {Colour::new(self.is_white_to_move)}
    pub fn colour_index(&self) -> usize {!self.is_white_to_move as usize /*self.colour_to_move().colour_index()*/}
    pub fn opponent_index(&self) -> usize {self.is_white_to_move as usize /*self.colour_to_move().opponent().colour_index()*/}
    pub fn get_king_square(&self) -> Square {self.side[self.colour_index()].king}
    pub fn get_king_treats(&self) -> i8 {
        return self.side[self.opponent_index()].attacks[self.get_king_square() as usize];
    }
    pub fn is_king_in_check(&self) -> bool {self.get_king_treats() != 0}

    pub fn make_move(&mut self, movee: Moves) {
        let colour = self.colour_to_move();
        let colour_index = self.colour_index();
        let opponent_index = self.opponent_index();
        match movee {
            Moves::KingCastling => {
                self.irreversable_state.push((NONE_TYPE, self.en_passant, self.castling, self.half_move));
                self.en_passant = u8::MAX;
                self.half_move += 1;
                if self.is_white_to_move {
                    self.make_castle(WHITE, 4, 7, 6, 5);
                    self.castling &= !(CASTLE_WHITE_QUEEN | CASTLE_WHITE_KING);
                } else {
                    self.make_castle(BLACK, 60, 63, 62, 61);
                    self.castling &= !(CASTLE_BLACK_QUEEN | CASTLE_BLACK_KING);
                }
            }
            Moves::QueenCastling => {
                self.irreversable_state.push((NONE_TYPE, self.en_passant, self.castling, self.half_move));
                self.en_passant = u8::MAX;
                self.half_move += 1;
                if self.is_white_to_move {
                    self.make_castle(WHITE, 4, 0, 2, 3);
                    self.castling &= !(CASTLE_WHITE_QUEEN | CASTLE_WHITE_KING);
                } else {
                    self.make_castle(BLACK, 60, 56, 58, 59);
                    self.castling &= !(CASTLE_BLACK_QUEEN | CASTLE_BLACK_KING);
                }
            }
            Moves::Move { start, target } => {
                let start_type = self.board[start as usize].get_type();
                let target_type = self.board[target as usize].get_type();
                
                self.irreversable_state.push((target_type, self.en_passant, self.castling, self.half_move));
                self.en_passant = u8::MAX;
                self.update_castling(start, target);
                
                self.side[colour_index].move_piece(start_type, start, target);

                self.board[start as usize] = NONE;
                self.remove_attack_and_update(start_type, colour, start);
                
                self.board[target as usize] = Piece::new(start_type, colour);

                if target_type == NONE_TYPE {
                    if start_type == PAWN
                    {self.half_move = 0}
                    else
                    {self.half_move += 1}
                    self.put_attack_and_update(start_type, colour, target);
                } else {
                    self.half_move = 0;
                    self.side[opponent_index].remove_piece(target_type, target);

                    self.piece_attack(target_type, colour.opponent(), target, -1);
                    self.piece_attack(start_type, colour, target, 1);
                }
            }
            Moves::Promotion { start, target, promotion_type } => {
                let target_type = self.board[target as usize].get_type();

                self.irreversable_state.push((target_type, self.en_passant, self.castling, self.half_move));
                self.en_passant = u8::MAX;
                self.half_move = 0;
                
                self.side[colour_index].remove_piece(PAWN, start);
                self.side[colour_index].pieces(promotion_type).push(target);

                self.board[start as usize] = NONE;
                self.remove_attack_and_update(PAWN, colour, start);
                
                self.board[target as usize] = Piece::new(promotion_type, colour);

                if target_type == NONE_TYPE {
                    self.put_attack_and_update(promotion_type, colour, target);
                } else {
                    self.update_castling_pawn_eats_and_promotion(target);
                    self.side[opponent_index].remove_piece(target_type, target);

                    self.piece_attack(target_type, colour.opponent(), target, -1);
                    self.piece_attack(promotion_type, colour, target, 1);
                }
            }
            Moves::DoublePush { start, target } => {
                self.irreversable_state.push((NONE_TYPE, self.en_passant, self.castling, self.half_move));
                self.en_passant = if self.is_white_to_move {target - 8} else {target + 8};
                self.half_move = 0;

                self.side[colour_index].move_piece(PAWN, start, target);

                self.board[target as usize] = Piece::new(PAWN, colour);
                self.put_attack_and_update(PAWN, colour, target);
                self.board[start as usize] = NONE;
                self.remove_attack_and_update(PAWN, colour, start);
            }
            Moves::EnPassant { start, target } => {
                let eaten_square = if self.is_white_to_move {target - 8} else {target + 8};

                self.irreversable_state.push((PAWN, self.en_passant, self.castling, self.half_move));
                self.en_passant = u8::MAX;
                self.half_move = 0;

                self.side[colour_index].move_piece(PAWN, start, target);
                self.side[opponent_index].remove_piece(PAWN, eaten_square);
                
                self.board[target as usize] = Piece::new(PAWN, colour);
                self.put_attack_and_update(PAWN, colour, target);
                self.board[start as usize] = NONE;
                self.remove_attack_and_update(PAWN, colour, start);
                self.board[eaten_square as usize] = NONE;
                self.remove_attack_and_update(PAWN, colour.opponent(), eaten_square);
            }
        }
        self.full_turn += self.is_white_to_move as u16; // false == 1
        self.is_white_to_move = !self.is_white_to_move;
    }

    fn make_castle(&mut self, colour: Colour, king: u8, rook: u8, king_target: u8, rook_target: u8) {
        self.side[colour.colour_index()].king = king_target;
        self.side[colour.colour_index()].move_piece(ROOK, rook, rook_target);
        
        self.board[rook as usize] = NONE;
        self.remove_attack_and_update(ROOK, colour, rook);
        self.board[king_target as usize] = Piece::new(KING, colour);
        self.put_attack_and_update(KING, colour, king_target);
        self.board[king as usize] = NONE;
        self.remove_attack_and_update(KING, colour, king);
        self.board[rook_target as usize] = Piece::new(ROOK, colour);
        self.put_attack_and_update(ROOK, colour, rook_target);
    }

    fn update_castling(&mut self, start: u8, target: u8) {
        for square in [start, target] {
            match square {
                0 => self.castling &= !CASTLE_WHITE_QUEEN,
                7 => self.castling &= !CASTLE_WHITE_KING,
                4 => self.castling &= !(CASTLE_WHITE_QUEEN + CASTLE_WHITE_KING),
                56 => self.castling &= !CASTLE_BLACK_QUEEN,
                63 => self.castling &= !CASTLE_BLACK_KING,
                60 => self.castling &= !(CASTLE_BLACK_QUEEN + CASTLE_BLACK_KING),
                _ => (),
            }
        }
    }
    fn update_castling_pawn_eats_and_promotion(&mut self, target: u8) {
        match target {
            0 => self.castling &= !CASTLE_WHITE_QUEEN,
            7 => self.castling &= !CASTLE_WHITE_KING,
            56 => self.castling &= !CASTLE_BLACK_QUEEN,
            63 => self.castling &= !CASTLE_BLACK_KING,
            _ => (),
        }
    }

    pub fn unmake_move(&mut self, movee: Moves) {
        self.is_white_to_move = !self.is_white_to_move;
        self.full_turn -= self.is_white_to_move as u16; // false == 1
        
        let colour = self.colour_to_move();
        let colour_index = self.colour_index();
        let opponent_index = self.opponent_index();
        let target_type;

        (target_type, self.en_passant, self.castling, self.half_move) = self.irreversable_state.pop().unwrap();
        
        match movee {
            Moves::KingCastling => {
                if self.is_white_to_move {
                    self.make_castle(WHITE, 6, 5, 4, 7);
                } else {
                    self.make_castle(BLACK, 62, 61, 60, 63);
                }
            }
            Moves::QueenCastling => {
                if self.is_white_to_move {
                    self.make_castle(WHITE, 2, 3, 4, 0);
                } else {
                    self.make_castle(BLACK, 58, 59, 60, 56);
                }
            }
            Moves::Move { start, target } => {
                let start_type = self.board[target as usize].get_type();
                
                if target_type == NONE_TYPE {
                    self.board[target as usize] = NONE;
                    self.remove_attack_and_update(start_type, colour, target);
                } else {
                    self.side[opponent_index].pieces(target_type).push(target);
                    
                    self.board[target as usize] = Piece::new(target_type, colour.opponent());
                    self.piece_attack(start_type, colour, target, -1);
                    self.piece_attack(target_type, colour.opponent(), target, 1);
                }
                self.side[colour_index].move_piece(start_type, target, start);
                
                self.board[start as usize] = Piece::new(start_type, colour);
                self.put_attack_and_update(start_type, colour, start);
            }
            Moves::Promotion { start, target, promotion_type } => {
                self.side[colour_index].remove_piece(promotion_type, target);
                self.side[colour_index].pieces(PAWN).push(start);

                self.board[start as usize] = Piece::new(PAWN, colour);
                self.put_attack_and_update(PAWN, colour, start);
                
                if target_type == NONE_TYPE {
                    self.board[target as usize] = NONE;
                    self.remove_attack_and_update(promotion_type, colour, target);
                } else {
                    self.side[opponent_index].pieces(target_type).push(target);

                    self.board[target as usize] = Piece::new(target_type, colour.opponent());
                    self.piece_attack(promotion_type, colour, target, -1);
                    self.piece_attack(target_type, colour.opponent(), target, 1);
                }
            }
            Moves::DoublePush { start, target } => {
                self.side[colour_index].move_piece(PAWN, target, start);
                
                self.board[start as usize] = Piece::new(PAWN, colour);
                self.put_attack_and_update(PAWN, colour, start);
                self.board[target as usize] = NONE;
                self.remove_attack_and_update(PAWN, colour, target);
            }
            Moves::EnPassant { start, target } => {
                let eaten_square = if self.is_white_to_move {target - 8} else {target + 8};

                self.side[colour_index].move_piece(PAWN, target, start);
                self.side[opponent_index].pieces(PAWN).push(eaten_square);
                
                self.board[start as usize] = Piece::new(PAWN, colour);
                self.put_attack_and_update(PAWN, colour, start);
                self.board[eaten_square as usize] = Piece::new(PAWN, colour.opponent());
                self.put_attack_and_update(PAWN, colour.opponent(), eaten_square);
                self.board[target as usize] = NONE;
                self.remove_attack_and_update(PAWN, colour, target);
            }
        }
    }
} 

pub mod utils {
    use super::*;

    pub fn is_black_square(square: Square) -> bool {
        (square + square / 8) % 2 == 0
    }
    pub fn number_from_0_9(number: char) -> usize {
        number.to_digit(10).unwrap() as usize
    }
    pub fn number_from_a_h(letter: char) -> usize {
        letter as usize - 'a' as usize
    }
    pub fn square_from_text(letter: char, number: char) -> Square {
        ((number_from_0_9(number)-1) * 8 + number_from_a_h(letter)) as u8
    }
    pub fn gen_move(chess: &Chess, text: &String) -> Moves {
        let mut chars = text.chars();
        let start = square_from_text(chars.next().unwrap(), chars.next().unwrap());
        let target = square_from_text(chars.next().unwrap(), chars.next().unwrap());
        let promotion = match chars.next() {
            None | Some('\n') => None,
            Some(symbol) => Some(Piece::from_symbol(symbol).get_type()),
        };
        if let Some(promotion_type) = promotion {
            return Moves::Promotion { start, target, promotion_type };
        }
        if chess.board[start as usize].is_type(KING) {
            if target as i8 - start as i8 == 2 {
                return Moves::KingCastling;
            } if target as i8 - start as i8 == -2 {
                return Moves::QueenCastling;
            }
        } else if chess.board[start as usize].is_type(PAWN) {
            if target as i8 - start as i8 == 16 {
                return Moves::DoublePush { start, target };
            } if target as i8 - start as i8 == -16 {
                return Moves::DoublePush { start, target };
            }
            if target == chess.en_passant {
                return Moves::EnPassant { start, target };
            }
        }
        return Moves::Move { start, target };
    }
}