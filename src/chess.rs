// SETTINGS
static DISPLAY: bool = true;
// max turn 30 * 50 = 1500
const MAX_DEPTH: usize = 3000;
const MAX_MOVES: usize = 218;
const START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const CASTLE_SPOILER: [bool; 64] = {
    let mut spoiler = [false; 64];
    spoiler[0]=true; spoiler[4]=true; spoiler[7]=true;
    spoiler[56]=true; spoiler[60]=true; spoiler[63]=true;
    spoiler
};
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
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Moves {
    Move{start: u8, target: u8},
    QueenCastling,
    KingCastling,
    Promotion{start: u8, target: u8, piece: PieceType},
    DoublePush{start: u8, target: u8},
    EnPassant{start: u8, target: u8},
}

#[derive(Debug, Clone, PartialEq)]
pub struct SideState {
    pub king_square: u8,
    pub pieces: Vec<u8>,
    pub control: [i8;64],
    pub slide_control: [i8;64],
}
impl SideState {
    fn add_piece_list(&mut self, piece_type: PieceType, square: u8) {
        if piece_type == KING
        {self.king_square = square}
        else
        {self.pieces.push(square)}
    }
    
    fn remove_piece_list(&mut self, square: u8) {
        if self.pieces.iter().position(|&n|n == square).is_none() {
            
            println!("{square}")}
        self.pieces.swap_remove(self.pieces.iter().position(|&n|n == square).unwrap());
    }
    
    fn update_pieces_list(&mut self, start: u8, target: u8) {
        if start == self.king_square {
            self.king_square = target
        } else {
            let idx = self.pieces.iter().position(|&s|s == start).unwrap();
            self.pieces[idx] = target
        }
    }
}
impl Chess {
    fn spread_control_direcion(&mut self, start: u8, colour: Colour, dir_index: usize, value: i8) {
        let side = if colour == WHITE {&mut self.white_side} else {&mut self.black_side};

        let dir = DIRECTION_OFFSETS[dir_index];
        let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[start as usize][dir_index]};
        
        for n in 0..dist_edge {
            let target = start as i8 + ((n+1) as i8*dir);
            let piece = self.board[target as usize];
            side.slide_control[target as usize] += value;
            side.control[target as usize] += value;
            
            if piece != NONE && piece != Piece::new(KING, colour.opponent()) {break}
        }
    }
    fn update_control_other_pieces(&mut self, piece: PieceType, square: u8, value: i8) {
        let mut white_control = self.white_side.slide_control[square as usize];
        let mut black_control = self.black_side.slide_control[square as usize];
        let (mut white, mut black) = (white_control > 0, black_control > 0);
        if piece == KING {
            if self.colour_turn == WHITE {
                black = false;
            } else {
                white = false;
            }
        }
        
        for dir_index in 0..8 {
            if !white && !black {break}

            let dir = DIRECTION_OFFSETS[dir_index];
            let dist_edge = unsafe {NUM_SQUARES_TO_EDGES[square as usize][dir_index]};
            
            for n in 0..dist_edge {
                let target = square as i8 + ((n+1) as i8*dir);
                let piece = self.board[target as usize];
                if piece == NONE {
                    continue;
                    if (!white || self.white_side.slide_control[target as usize] == 0)
                    && (!black || self.black_side.slide_control[target as usize] == 0) {
                        break;
                    }
                }
                else if piece.is_colour(WHITE) {
                    let piece_type = piece.get_type();
                    if !white
                    || !piece_type.is_sliding()
                    || !piece_type.get_sliding_indices().contains(&dir_index) {
                        break
                    }
                    white_control -= 1;
                    white = white_control > 0;
                    self.spread_control_direcion(square, WHITE, DIRECTION_OPPOST_INDEX[dir_index], value);
                } else { 
                    let piece_type = piece.get_type();
                    if !black
                    || !piece_type.is_sliding()
                    || !piece_type.get_sliding_indices().contains(&dir_index) {
                        break
                    }
                    black_control -= 1;
                    black = black_control > 0;
                    self.spread_control_direcion(square, BLACK, DIRECTION_OPPOST_INDEX[dir_index], value);
                }
                break;
            }
        }
        
    }
    fn update_control_piece(&mut self, colour: Colour, piece: PieceType, square: u8, value: i8) {
        if piece.is_sliding() {self.update_control_sliding(piece, colour, square, value)}
        else {
            let side = if colour == WHITE {&mut self.white_side} else {&mut self.black_side};

            let control_squares = match piece {
                KING => unsafe {&KING_CONTROL[square as usize]},
                KNIGHT => unsafe {&KNIGHT_CONTROL[square as usize]},
                _ if colour == WHITE =>  unsafe {&WHITE_PAWN_CONTROL[square as usize]},
                _ =>  unsafe {&BLACK_PAWN_CONTROL[square as usize]},
            };
            for &square in control_squares {
                side.control[square as usize] += value;
            }
        }
    }
    fn update_control_sliding(&mut self, piece: PieceType, colour: Colour, square: u8, value: i8) {
        for dir_index in piece.get_sliding_indices() {
            self.spread_control_direcion(square, colour, dir_index, value);
        }
    }
    pub fn update_control(&mut self, colour: Colour, piece: PieceType, square: u8, value: i8) {
        self.update_control_piece(colour, piece, square, value);
        self.update_control_other_pieces(piece, square, -value);
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
    pub board: [Piece;64],
    pub en_passant: u8,
    pub castling: u8,
    pub half_move: u16,
    pub full_turn: u16,
    pub colour_turn: Colour,
    pub white_side: SideState,
    pub black_side: SideState,
    // eaten piece | en passant | castlilg | half move clock
    pub irreversable_state: Vec<(PieceType, u8, u8, u16)>,
}


impl Chess {
    pub fn new() -> Chess {Chess::build(START_POSITION)}
    
    pub fn make_move(&mut self, movee: Moves) {
        if self.colour_turn == WHITE { // White
            match movee {
                Moves::KingCastling | Moves::QueenCastling => {
                    if movee == Moves::KingCastling 
                    {self.make_white_castle(4, 7, 6, 5)}
                    else
                    {self.make_white_castle(4, 0, 2, 3)}

                    self.irreversable_state.push((NONE_TYPE, self.en_passant, self.castling, self.half_move));
                    self.en_passant = u8::MAX;
                    self.castling &= !CASTLE_WHITE_QUEEN;
                    self.castling &= !CASTLE_WHITE_KING;
                    self.half_move += 1;
                }
                Moves::Move { start, target } => {
                    let start_piece = self.board[start as usize].get_type();
                    let target_piece = self.board[target as usize].get_type();
                    
                    self.irreversable_state.push((target_piece, self.en_passant, self.castling, self.half_move));
                    self.en_passant = u8::MAX;
                    self.update_castling(start, target);
                    
                    self.white_side.update_pieces_list(start, target);
                    
                    if target_piece == NONE_TYPE {
                        self.update_control(WHITE, start_piece, target, 1);
                        if start_piece == PAWN {self.half_move = 0}
                        else {self.half_move += 1}
                    } else {
                        self.black_side.remove_piece_list(target);

                        self.update_control_piece(BLACK, target_piece, target, -1);
                        self.update_control_piece(WHITE, start_piece, target, 1);
                        self.half_move = 0;
                    }
                    self.board[target as usize] = self.board[start as usize];
                    self.board[start as usize] = NONE;
                    self.update_control(WHITE, start_piece, start, -1);
                }
                Moves::Promotion { start, target, piece } => {
                    let target_piece = self.board[target as usize].get_type();

                    self.irreversable_state.push((target_piece, self.en_passant, self.castling, self.half_move));
                    self.en_passant = u8::MAX;
                    self.half_move = 0;
                    
                    self.white_side.update_pieces_list(start, target);

                    self.update_control(WHITE, PAWN, start, -1);
                    self.board[target as usize] = Piece::new(piece, WHITE);
                    self.board[start as usize] = NONE;

                    if target_piece == NONE_TYPE {
                        self.update_control(WHITE, piece, target, 1);
                    } else {
                        self.update_control_piece(BLACK, target_piece, target, -1);
                        self.update_control_piece(WHITE, piece, target, 1);
                        self.update_castling_white_pawn_eats_and_promotion(target);
                        self.black_side.remove_piece_list(target);
                    }
                }
                Moves::DoublePush { start, target } => {
                    self.irreversable_state.push((NONE_TYPE, self.en_passant, self.castling, self.half_move));
                    self.en_passant = start + 8;
                    self.half_move = 0;

                    self.white_side.update_pieces_list(start, target);

                    self.update_control(WHITE, PAWN, target, 1);
                    self.board[target as usize] = Piece::new(PAWN, WHITE);
                    self.board[start as usize] = NONE;
                    self.update_control(WHITE, PAWN, start, -1);
                }
                Moves::EnPassant { start, target } => {
                    let eaten_square = target - 8;

                    self.irreversable_state.push((PAWN, self.en_passant, self.castling, self.half_move));
                    self.en_passant = u8::MAX;
                    self.half_move = 0;

                    self.white_side.update_pieces_list(start, target);
                    self.black_side.remove_piece_list(eaten_square);
                    
                    self.update_control(WHITE, PAWN, target, 1);
                    self.board[target as usize] = Piece::new(PAWN, WHITE);
                    self.board[start as usize] = NONE;
                    self.update_control(WHITE, PAWN, start, -1);
                    self.update_control(BLACK, PAWN, eaten_square, -1);
                    self.board[eaten_square as usize] = NONE;
                }
            }
            self.colour_turn = BLACK;
        }
        else { // Black
            match movee {
                Moves::KingCastling | Moves::QueenCastling => {
                    if movee == Moves::KingCastling 
                    {self.make_black_castle(60, 63, 62, 61)}
                    else
                    {self.make_black_castle(60, 56, 58, 59)}

                    self.irreversable_state.push((NONE_TYPE, self.en_passant, self.castling, self.half_move));
                    self.en_passant = u8::MAX;
                    self.castling &= !CASTLE_BLACK_QUEEN;
                    self.castling &= !CASTLE_BLACK_KING;
                    self.half_move += 1;
                }
                Moves::Move { start, target } => {
                    let start_piece = self.board[start as usize].get_type();
                    let target_piece = self.board[target as usize].get_type();
                    
                    self.irreversable_state.push((target_piece, self.en_passant, self.castling, self.half_move));
                    self.en_passant = u8::MAX;
                    self.update_castling(start, target);
                    
                    self.black_side.update_pieces_list(start, target);
                    
                    if target_piece == NONE_TYPE {
                        self.update_control(BLACK, start_piece, target, 1);
                        if start_piece == PAWN {self.half_move = 0}
                        else {self.half_move += 1}
                    } else {
                        self.white_side.remove_piece_list(target);
                        
                        self.update_control_piece(WHITE, target_piece, target, -1);
                        self.update_control_piece(BLACK, start_piece, target, 1);
                        self.half_move = 0;
                    }
                    self.board[target as usize] = self.board[start as usize];
                    self.board[start as usize] = NONE;
                    self.update_control(BLACK, start_piece, start, -1);
                }
                Moves::Promotion { start, target, piece } => {
                    let target_piece = self.board[target as usize].get_type();

                    self.irreversable_state.push((target_piece, self.en_passant, self.castling, self.half_move));
                    self.en_passant = u8::MAX;
                    self.half_move = 0;
                    
                    self.black_side.update_pieces_list(start, target);

                    self.update_control(BLACK, PAWN, start, -1);
                    self.board[target as usize] = Piece::new(piece, BLACK);
                    self.board[start as usize] = NONE;

                    if target_piece == NONE_TYPE {
                        self.update_control(BLACK, piece, target, 1);
                    } else {
                        self.update_control_piece(WHITE, target_piece, target, -1);
                        self.update_control_piece(BLACK, piece, target, 1);
                        self.update_castling_black_pawn_eats_and_promotion(target);
                        self.white_side.remove_piece_list(target);
                    }
                }
                Moves::DoublePush { start, target } => {
                    self.irreversable_state.push((NONE_TYPE, self.en_passant, self.castling, self.half_move));
                    self.en_passant = start - 8;
                    self.half_move = 0;

                    self.black_side.update_pieces_list(start, target);

                    self.update_control(BLACK, PAWN, target, 1);
                    self.board[target as usize] = Piece::new(PAWN, BLACK);
                    self.board[start as usize] = NONE;
                    self.update_control(BLACK, PAWN, start, -1);
                }
                Moves::EnPassant { start, target } => {
                    let eaten_square = target + 8;

                    self.irreversable_state.push((PAWN, self.en_passant, self.castling, self.half_move));
                    self.en_passant = u8::MAX;
                    self.half_move = 0;

                    self.black_side.update_pieces_list(start, target);
                    self.white_side.remove_piece_list(eaten_square);
                    
                    self.update_control(BLACK, PAWN, target, 1);
                    self.board[target as usize] = Piece::new(PAWN, BLACK);
                    self.board[start as usize] = NONE;
                    self.update_control(BLACK, PAWN, start, -1);
                    self.update_control(WHITE, PAWN, eaten_square, -1);
                    self.board[eaten_square as usize] = NONE;
                }
            }
            self.colour_turn = WHITE;
            self.full_turn += 1;
        }
    }

    fn make_white_castle(&mut self, king: u8, rook: u8, king_target: u8, rook_target: u8) {
        self.white_side.king_square = king_target;
        self.white_side.update_pieces_list(rook, rook_target);
        
        self.update_control(WHITE, ROOK, rook, -1);
        self.board[rook as usize] = NONE;
        self.board[king_target as usize] = Piece::new(KING, WHITE);
        self.update_control(WHITE, KING, king_target, 1);
        self.update_control(WHITE, KING, king, -1);
        self.board[king as usize] = NONE;
        self.board[rook_target as usize] = Piece::new(ROOK, WHITE);
        self.update_control(WHITE, ROOK, rook_target, 1);
    }
    fn make_black_castle(&mut self, king: u8, rook: u8, king_target: u8, rook_target: u8) {
        self.black_side.king_square = king_target;
        self.black_side.update_pieces_list(rook, rook_target);
        
        self.update_control(BLACK, ROOK, rook, -1);
        self.board[rook as usize] = NONE;
        self.board[king_target as usize] = Piece::new(KING, BLACK);
        self.update_control(BLACK, KING, king_target, 1);
        self.update_control(BLACK, KING, king, -1);
        self.board[king as usize] = NONE;
        self.board[rook_target as usize] = Piece::new(ROOK, BLACK);
        self.update_control(BLACK, ROOK, rook_target, 1);
    }

    fn update_castling(&mut self, start: u8, target: u8) {
        if (CASTLE_SPOILER[start as usize]
        || CASTLE_SPOILER[target as usize])
        && self.castling != CASTLE_NONE {
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
    }
    fn update_castling_white_pawn_eats_and_promotion(&mut self, target: u8) {
        match target {
            56 => self.castling &= !CASTLE_BLACK_QUEEN,
            63 => self.castling &= !CASTLE_BLACK_KING,
            _ => (),
        }
    }
    fn update_castling_black_pawn_eats_and_promotion(&mut self, target: u8) {
        match target {
            0 => self.castling &= !CASTLE_WHITE_QUEEN,
            7 => self.castling &= !CASTLE_WHITE_KING,
            _ => (),
        }
    }

    pub fn unmake_move(&mut self, movee: Moves) {
        let target_piece;
        (target_piece, self.en_passant, self.castling, self.half_move) = self.irreversable_state.pop().unwrap();
        if self.colour_turn == BLACK { // White
            self.colour_turn = WHITE;
            match movee {
                Moves::KingCastling | Moves::QueenCastling => {
                    if movee == Moves::KingCastling 
                    {self.make_white_castle(6, 5, 4, 7)}
                    else
                    {self.make_white_castle(2, 3, 4, 0)}
                }
                Moves::Move { start, target } => {
                    let start_piece = self.board[target as usize].get_type();
                    
                    self.white_side.update_pieces_list(target, start);
                    
                    self.update_control(WHITE, start_piece, start, 1);
                    self.board[start as usize] = self.board[target as usize];
                    
                    if target_piece == NONE_TYPE {
                        self.board[target as usize] = NONE;
                        self.update_control(WHITE, start_piece, target, -1);
                    } else {
                        self.board[target as usize] = Piece::new(target_piece, BLACK);
                        self.update_control_piece(WHITE, start_piece, target, -1);
                        self.update_control_piece(BLACK, target_piece, target, 1);

                        self.black_side.pieces.push(target);
                    }
                }
                Moves::Promotion { start, target, piece } => {
                    self.white_side.update_pieces_list(target, start);

                    self.update_control(WHITE, PAWN, start, 1);
                    self.board[start as usize] = Piece::new(PAWN, WHITE);
                    
                    if target_piece == NONE_TYPE {
                        self.board[target as usize] = NONE;
                        self.update_control(WHITE, piece, target, -1);
                    } else {
                        self.board[target as usize] = Piece::new(target_piece, BLACK);
                        self.update_control_piece(WHITE, piece, target, -1);
                        self.update_control_piece(BLACK, target_piece, target, 1);
                        self.black_side.pieces.push(target);
                    }
                }
                Moves::DoublePush { start, target } => {
                    self.white_side.update_pieces_list(target, start);

                    self.update_control(WHITE, PAWN, start, 1);
                    self.board[start as usize] = Piece::new(PAWN, WHITE);
                    self.board[target as usize] = NONE;
                    self.update_control(WHITE, PAWN, target, -1);
                }
                Moves::EnPassant { start, target } => {
                    let eaten_square = target - 8;

                    self.white_side.update_pieces_list(target, start);
                    self.black_side.pieces.push(eaten_square);
                    
                    self.update_control(WHITE, PAWN, start, 1);
                    self.board[start as usize] = Piece::new(PAWN, WHITE);
                    self.board[eaten_square as usize] = Piece::new(target_piece, BLACK);
                    self.update_control(BLACK, PAWN, eaten_square, 1);
                    self.update_control(WHITE, PAWN, target, -1);
                    self.board[target as usize] = NONE;
                }
            }
        }
        else { // Black
            self.colour_turn = BLACK;
            self.full_turn -= 1;
            match movee {
                Moves::KingCastling | Moves::QueenCastling => {
                    if movee == Moves::KingCastling 
                    {self.make_black_castle(62, 61, 60, 63)}
                    else
                    {self.make_black_castle(58, 59, 60, 56)}
                }
                Moves::Move { start, target } => {
                    let start_piece = self.board[target as usize].get_type();
                    
                    self.black_side.update_pieces_list(target, start);
                    
                    self.update_control(BLACK, start_piece, start, 1);
                    self.board[start as usize] = self.board[target as usize];
                    
                    if target_piece == NONE_TYPE {
                        self.board[target as usize] = NONE;
                        self.update_control(BLACK, start_piece, target, -1);
                    } else {
                        self.board[target as usize] = Piece::new(target_piece, WHITE);
                        self.update_control_piece(BLACK, start_piece, target, -1);
                        self.update_control_piece(WHITE, target_piece, target, 1);

                        self.white_side.pieces.push(target);
                    }
                }
                Moves::Promotion { start, target, piece } => {
                    self.black_side.update_pieces_list(target, start);

                    self.update_control(BLACK, PAWN, start, 1);
                    self.board[start as usize] = Piece::new(PAWN, BLACK);
                    
                    if target_piece == NONE_TYPE {
                        self.board[target as usize] = NONE;
                        self.update_control(BLACK, piece, target, -1);
                    } else {
                        self.board[target as usize] = Piece::new(target_piece, WHITE);
                        self.update_control_piece(BLACK, piece, target, -1);
                        self.update_control_piece(WHITE, target_piece, target, 1);
                        self.white_side.pieces.push(target);
                    }
                }
                Moves::DoublePush { start, target } => {
                    self.black_side.update_pieces_list(target, start);

                    self.update_control(BLACK, PAWN, start, 1);
                    self.board[start as usize] = Piece::new(PAWN, BLACK);
                    self.board[target as usize] = NONE;
                    self.update_control(BLACK, PAWN, target, -1);
                }
                Moves::EnPassant { start, target } => {
                    let eaten_square = target + 8;

                    self.black_side.update_pieces_list(target, start);
                    self.white_side.pieces.push(eaten_square);
                    
                    self.update_control(BLACK, PAWN, start, 1);
                    self.board[start as usize] = Piece::new(PAWN, BLACK);
                    self.board[eaten_square as usize] = Piece::new(target_piece, WHITE);
                    self.update_control(WHITE, PAWN, eaten_square, 1);
                    self.update_control(BLACK, PAWN, target, -1);
                    self.board[target as usize] = NONE;
                }
            }
        }
    }
} 

pub mod utils {
    use super::*;

    pub fn number_from_0_9(number: char) -> usize {
        number.to_digit(10).unwrap() as usize
    }
    pub fn number_from_a_h(letter: char) -> usize {
        letter as usize - 'a' as usize
    }
    pub fn square_from_text(letter: char, number: char) -> u8 {
        ((number_from_0_9(number)-1) * 8 + number_from_a_h(letter)) as u8
    }
    pub fn gen_move(chess: &Chess, text: &String) -> Moves {
        let mut chars = text.chars();
        let start = square_from_text(chars.next().unwrap(), chars.next().unwrap());
        let target = square_from_text(chars.next().unwrap(), chars.next().unwrap());
        let prom = match chars.next() {
            None | Some('\n') => None,
            Some(symbol) => Some(Piece::from_symbol(symbol).get_type()),
        };
        if let Some(piece) = prom {
            return Moves::Promotion { start, target, piece };
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