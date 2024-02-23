
// max turn 30 * 50 = 1500
const MAX_DEPTH: usize = 3000;
//const MAX_MOVES: usize = 218;

mod fen;
mod display;
pub mod r#move;
pub mod piece;
pub mod legal_moves;
pub mod perft;
pub mod zobrist;

pub use fen::*;
pub use piece::*;

use castle::*;
use legal_moves::*;
pub use super::*;
pub use r#move::*;
pub use zobrist::*;
pub use utils::*;

pub type Square = i8;

pub mod castle {
    pub const CASTLE_NONE: u8 = 0;
    pub const CASTLE_WHITE_QUEEN: u8 = 1;
    pub const CASTLE_WHITE_KING: u8 = 2;
    pub const CASTLE_BLACK_QUEEN: u8 = 4;
    pub const CASTLE_BLACK_KING: u8 = 8;
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
impl Chess {
    fn pieces(&mut self, colour_index: usize, piece_type: PieceType) -> &mut Vec<Square> {
        match piece_type {
            KING => unreachable!(),
            PAWN => &mut self.side[colour_index].pawns,
            KNIGHT => &mut self.side[colour_index].knights,
            BISHOP => &mut self.side[colour_index].bishops,
            ROOK => &mut self.side[colour_index].rooks,
            QUEEN => &mut self.side[colour_index].queens,
            _ => unreachable!(),
        }
    }
    
    fn new_piece(&mut self, colour_index: usize, piece_type: PieceType, square: Square) {
        match piece_type {
            KING => self.side[colour_index].king = square,
            _ => self.pieces(colour_index, piece_type).push(square),
        }
        self.piece_hash(piece_type, colour_index, square);
    }

    fn add_piece(&mut self, colour_index: usize, piece_type: PieceType, square: Square) {
        self.pieces(colour_index, piece_type).push(square);
        self.piece_hash(piece_type, colour_index, square);
    }
    
    fn remove_piece(&mut self, colour_index: usize, piece_type: PieceType, square: Square) {
        let pieces = self.pieces(colour_index, piece_type);
        pieces.swap_remove(pieces.iter().position(|&p|p == square).unwrap());
        self.piece_hash(piece_type, colour_index, square);
    }
    
    fn move_piece(&mut self, colour_index: usize, piece_type: PieceType, start: Square, target: Square) {
        match piece_type {
            KING => self.side[colour_index].king = target,
            _ => {
                let pieces = self.pieces(colour_index, piece_type);
                let index = pieces.iter().position(|&p|p == start).unwrap();
                pieces[index] = target;
            }
        }
        self.piece_hash(piece_type, colour_index, start);
        self.piece_hash(piece_type, colour_index, target);
    }

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
    fn put_attack_and_update(&mut self, piece_type: PieceType, colour: Colour, square: Square) {
        self.piece_attack(piece_type, colour, square, 1);
        self.update_attack_pieces(piece_type, colour, square, -1);
    }
    fn remove_attack_and_update(&mut self, piece_type: PieceType, colour: Colour, square: Square) {
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
    board: [Piece; 64],
    pub en_passant: Square,
    pub castling: u8,
    pub half_move: u16,
    pub full_turn: u16,
    is_white_to_move: bool,
    pub side: [SideState; 2],
    // eaten piece | en passant | castlilg | half move clock | hash
    pub irreversable_state: Vec<(PieceType, Square, u8, u16, Hash)>,
    hash: Hash,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChessOutcome {Draw, WhiteWinner, BlackWinner}

impl Chess {
    pub fn new() -> Chess {
        legal_moves::precompute();
        
        let mut chess = Chess {
            board: [NONE; 64],
            en_passant: -1,
            castling: CASTLE_NONE,
            half_move: 0,
            full_turn: 1,
            is_white_to_move: true,
            side: [Default::default(), Default::default()],
            irreversable_state: Vec::new(),
            //moves_history: Vec::new(),
            hash: 0,
        };
        chess.new_piece(0, KING, 15);
        chess.new_piece(1, KING, 49);
        chess.irreversable_state.reserve_exact(MAX_DEPTH);
        //chess.moves_history.reserve_exact(MAX_MOVES);
        chess
    }
    pub fn start_position() -> Chess {Chess::build(START_POSITION)}
    pub fn position(num: usize) -> Chess {
        Chess::build([
            START_POSITION,
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        ][num-1])
    }
    pub fn hash(&self) -> Hash {
        let hash = self.castling_hash(self.hash);
        self.en_passant_hash(hash)
    }
    pub fn is_white_to_move(&self) -> bool { self.is_white_to_move }
    fn set_turn(&mut self, is_white_to_move: bool) {
        if self.is_white_to_move != is_white_to_move {
            self.black_turn_hash();
        }
        self.is_white_to_move = is_white_to_move;
    }
    fn get_repetitions(&self) -> u16 {
        let hash = self.hash();
        let positions = self.half_move;
        self.irreversable_state.iter().rev().take(positions as usize).skip(3).step_by(2).take_while(|el|{
            self.castling == el.2
        }).filter(|el|el.4 == hash).count() as u16
    }
    pub fn board(&self, square: Square) -> Piece { self.board[square as usize] }
    pub fn colour_to_move(&self) -> Colour { Colour::new(self.is_white_to_move) }
    pub fn colour_index(&self) -> usize { !self.is_white_to_move as usize /*self.colour_to_move().colour_index()*/ }
    pub fn opponent_index(&self) -> usize { self.is_white_to_move as usize /*self.colour_to_move().opponent().colour_index()*/ }
    fn get_king_square(&self) -> Square { self.side[self.colour_index()].king }
    fn get_king_treats(&self) -> i8 { self.side[self.opponent_index()].attacks[self.get_king_square() as usize] }
    fn is_king_in_check(&self) -> bool { self.get_king_treats() != 0 }
    pub fn is_finished(&self, moves: &Vec<Move>) -> bool {
        moves.len() == 0 || self.get_repetitions() >= 1 || self.half_move >= 100
    }
    pub fn is_finished_for_real(&self, moves: &Vec<Move>) -> bool {
        moves.len() == 0 || self.get_repetitions() >= 3 || self.half_move >= 100
    }
    pub fn get_outcome(&self, moves: &Vec<Move>) -> ChessOutcome {
        if self.is_king_in_check() && moves.len() == 0 {
            if self.is_white_to_move {return ChessOutcome::BlackWinner}
            return ChessOutcome::WhiteWinner;
        }
        return ChessOutcome::Draw;
    }

    pub fn make_move(&mut self, r#move: Move) {
        let colour = self.colour_to_move();
        let colour_index = self.colour_index();
        let opponent_index = self.opponent_index();
        

        match r#move.flag() {
            CASTLE_FLAG => {
                self.irreversable_state.push((NONE_TYPE, self.en_passant, self.castling, self.half_move, self.hash()));
                self.en_passant = -1;
                self.half_move += 1;
                if self.is_white_to_move {
                    self.castling &= !(CASTLE_WHITE_QUEEN | CASTLE_WHITE_KING);
                    if r#move.is_king_castling() {
                        self.make_castle(WHITE, 4, 7, 6, 5);
                    } else {
                        self.make_castle(WHITE, 4, 0, 2, 3);
                    }
                } else {
                    self.castling &= !(CASTLE_BLACK_QUEEN | CASTLE_BLACK_KING);
                    if r#move.is_king_castling() {
                        self.make_castle(BLACK, 60, 63, 62, 61);
                    } else {
                        self.make_castle(BLACK, 60, 56, 58, 59);
                    }
                }
            }
            NO_FLAG => {
                let (start, target) = (r#move.start(), r#move.target());
                let start_type = self.board[start as usize].get_type();
                let target_type = self.board[target as usize].get_type();
                
                self.irreversable_state.push((target_type, self.en_passant, self.castling, self.half_move, self.hash()));
                self.en_passant = -1;
                self.update_castling(start, target);
                
                self.move_piece(colour_index, start_type, start, target);

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
                    self.remove_piece(opponent_index, target_type, target);

                    self.piece_attack(target_type, colour.opponent(), target, -1);
                    self.piece_attack(start_type, colour, target, 1);
                }
            }
            DOUBLE_PUSH_FLAG => {
                let (start, target) = (r#move.start(), r#move.target());
                self.irreversable_state.push((NONE_TYPE, self.en_passant, self.castling, self.half_move, self.hash()));
                self.en_passant = if self.is_white_to_move {target - 8} else {target + 8};
                self.half_move = 0;
                
                self.move_piece(colour_index, PAWN, start, target);

                self.board[target as usize] = Piece::new(PAWN, colour);
                self.put_attack_and_update(PAWN, colour, target);
                self.board[start as usize] = NONE;
                self.remove_attack_and_update(PAWN, colour, start);
            }
            EN_PASSANT_FLAG => {
                let (start, target) = (r#move.start(), r#move.target());
                let eaten_square = if self.is_white_to_move {target - 8} else {target + 8};

                self.irreversable_state.push((PAWN, self.en_passant, self.castling, self.half_move, self.hash()));
                self.en_passant = -1;
                self.half_move = 0;

                self.move_piece(colour_index, PAWN, start, target);
                self.remove_piece(opponent_index, PAWN, eaten_square);
                
                self.board[target as usize] = Piece::new(PAWN, colour);
                self.put_attack_and_update(PAWN, colour, target);
                self.board[start as usize] = NONE;
                self.remove_attack_and_update(PAWN, colour, start);
                self.board[eaten_square as usize] = NONE;
                self.remove_attack_and_update(PAWN, colour.opponent(), eaten_square);
            }
            _ => { // Promotion
                let (start, target) = (r#move.start(), r#move.target());
                let promotion_type = r#move.promotion_type();
                let target_type = self.board[target as usize].get_type();

                self.irreversable_state.push((target_type, self.en_passant, self.castling, self.half_move, self.hash()));
                self.en_passant = -1;
                self.half_move = 0;
                
                self.remove_piece(colour_index, PAWN, start);
                self.add_piece(colour_index, promotion_type, target);

                self.board[start as usize] = NONE;
                self.remove_attack_and_update(PAWN, colour, start);
                
                self.board[target as usize] = Piece::new(promotion_type, colour);

                if target_type == NONE_TYPE {
                    self.put_attack_and_update(promotion_type, colour, target);
                } else {
                    self.update_castling_pawn_eats_and_promotion(target);
                    self.remove_piece(opponent_index, target_type, target);

                    self.piece_attack(target_type, colour.opponent(), target, -1);
                    self.piece_attack(promotion_type, colour, target, 1);
                }
            }
        }
        
        self.black_turn_hash();
        self.full_turn += self.is_white_to_move as u16; // false == 1
        self.is_white_to_move = !self.is_white_to_move;
    }

    fn make_castle(&mut self, colour: Colour, king: Square, rook: Square, king_target: Square, rook_target: Square) {
        self.move_piece(colour.colour_index(), KING, king, king_target);
        self.move_piece(colour.colour_index(), ROOK, rook, rook_target);
        
        self.board[rook as usize] = NONE;
        self.remove_attack_and_update(ROOK, colour, rook);
        self.board[king_target as usize] = Piece::new(KING, colour);
        self.put_attack_and_update(KING, colour, king_target);
        self.board[king as usize] = NONE;
        self.remove_attack_and_update(KING, colour, king);
        self.board[rook_target as usize] = Piece::new(ROOK, colour);
        self.put_attack_and_update(ROOK, colour, rook_target);
    }

    fn update_castling(&mut self, start: Square, target: Square) {
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
    fn update_castling_pawn_eats_and_promotion(&mut self, target: Square) {
        match target {
            0 => self.castling &= !CASTLE_WHITE_QUEEN,
            7 => self.castling &= !CASTLE_WHITE_KING,
            56 => self.castling &= !CASTLE_BLACK_QUEEN,
            63 => self.castling &= !CASTLE_BLACK_KING,
            _ => (),
        }
    }

    pub fn unmake_move(&mut self, r#move: Move) {
        let (start, target) = (r#move.start(), r#move.target());
        self.is_white_to_move = !self.is_white_to_move;
        self.full_turn -= self.is_white_to_move as u16; // false == 1
        
        let colour = self.colour_to_move();
        let colour_index = self.colour_index();
        let opponent_index = self.opponent_index();
        let target_type;
        self.black_turn_hash();
        
        (target_type, self.en_passant, self.castling, self.half_move, _) = self.irreversable_state.pop().unwrap();
        
        match r#move.flag() {
            CASTLE_FLAG => {
                if self.is_white_to_move {
                    if r#move.is_king_castling() {
                        self.make_castle(WHITE, 6, 5, 4, 7);
                    } else {
                        self.make_castle(WHITE, 2, 3, 4, 0);
                    }
                } else {
                    if r#move.is_king_castling() {
                        self.make_castle(BLACK, 62, 61, 60, 63);
                    } else {
                        self.make_castle(BLACK, 58, 59, 60, 56);
                    }
                }
            }
            NO_FLAG => {
                let start_type = self.board[target as usize].get_type();
                
                if target_type == NONE_TYPE {
                    self.board[target as usize] = NONE;
                    self.remove_attack_and_update(start_type, colour, target);
                } else {
                    self.add_piece(opponent_index, target_type, target);
                    
                    self.board[target as usize] = Piece::new(target_type, colour.opponent());
                    self.piece_attack(start_type, colour, target, -1);
                    self.piece_attack(target_type, colour.opponent(), target, 1);
                }
                self.move_piece(colour_index, start_type, target, start);
                
                self.board[start as usize] = Piece::new(start_type, colour);
                self.put_attack_and_update(start_type, colour, start);
            }
            DOUBLE_PUSH_FLAG => {
                self.move_piece(colour_index, PAWN, target, start);
                
                self.board[start as usize] = Piece::new(PAWN, colour);
                self.put_attack_and_update(PAWN, colour, start);
                self.board[target as usize] = NONE;
                self.remove_attack_and_update(PAWN, colour, target);
            }
            EN_PASSANT_FLAG => {
                let eaten_square = if self.is_white_to_move {target - 8} else {target + 8};

                self.move_piece(colour_index, PAWN, target, start);
                self.add_piece(opponent_index, PAWN, eaten_square);
                
                self.board[start as usize] = Piece::new(PAWN, colour);
                self.put_attack_and_update(PAWN, colour, start);
                self.board[eaten_square as usize] = Piece::new(PAWN, colour.opponent());
                self.put_attack_and_update(PAWN, colour.opponent(), eaten_square);
                self.board[target as usize] = NONE;
                self.remove_attack_and_update(PAWN, colour, target);
            }
            _ => {
                let promotion_type = r#move.promotion_type();
                self.remove_piece(colour_index, promotion_type, target);
                self.add_piece(colour_index, PAWN, start);

                self.board[start as usize] = Piece::new(PAWN, colour);
                self.put_attack_and_update(PAWN, colour, start);
                
                if target_type == NONE_TYPE {
                    self.board[target as usize] = NONE;
                    self.remove_attack_and_update(promotion_type, colour, target);
                } else {
                    self.add_piece(opponent_index, target_type, target);

                    self.board[target as usize] = Piece::new(target_type, colour.opponent());
                    self.piece_attack(promotion_type, colour, target, -1);
                    self.piece_attack(target_type, colour.opponent(), target, 1);
                }
            }
        }
    }
} 

pub mod utils {
    use super::*;
    
    pub fn is_black_square(square: Square) -> bool {
        (square + square / 8) % 2 == 0
    }
    
    pub fn square_to_text(square: Square) -> String {
        format!("{}{}", ('a' as Square + square % 8) as u8 as char, (square / 8) + 1)
    }
    pub fn square_from_text(letter: char, number: char) -> Square {
        let rank = number.to_digit(10).unwrap() -1;
        let file = letter as u32 - 'a' as u32;
        return (rank * 8 + file) as Square;
    }

}
