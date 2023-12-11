// SETTINGS
static DISPLAY: bool = true;
// max turn 30 * 50 = 1500
const MAX_DEPTH: usize = 3000;
const MAX_MOVES: usize = 218;
const START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

mod fen;
pub mod legal_moves;
mod display;

use piece::*;
use castle::*;


pub mod piece {
    pub const NONE: u8=0;
    pub const KING: u8=1;
    pub const PAWN: u8=2;
    pub const KNIGHT: u8=3;
    pub const BISHOP: u8=4;
    pub const ROCK: u8=5;
    pub const QUEEN: u8=6;

    pub const WHITE: u8=8;
    pub const BLACK: u8=16;
    pub fn look(piece: u8) -> char {
        if piece & WHITE != 0 {
            [' ','K','P','N','B','R','Q'][(piece ^ WHITE) as usize]
        } else {
            [' ','k','p','n','b','r','q'][(piece ^ BLACK) as usize]
        }
    }
    pub fn code(symbol: char) -> u8 {
        let piece = match symbol.to_ascii_lowercase() {
            'p' => PAWN, 'n' => KNIGHT, 'b' => BISHOP,
            'r' => ROCK, 'q' => QUEEN, 'k' => KING,
            _ => NONE,
        };
        piece | if symbol.is_ascii_uppercase() {WHITE} else {BLACK}
    }
    pub fn is_sliding_piece(piece: u8) -> bool {
        piece_type(piece) >= 4
    } 
    pub fn is_type(piece: u8, t: u8) -> bool {piece_type(piece) == t}
    fn piece_type(piece: u8) -> u8 {piece & !(WHITE + BLACK)}
    pub fn is_colour(piece: u8, colour: u8) -> bool {piece & colour != 0}
}
mod castle {
    pub const CASTLE_NONE: u8=0;
    pub const CASTLE_W_K: u8=1;
    pub const CASTLE_W_Q: u8=2;
    pub const CASTLE_B_K: u8=4;
    pub const CASTLE_B_Q: u8=8;
}
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Move {
    pub start_square: u8,
    pub target_square: u8,
    pub promotion: u8
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
#[derive(Debug)]
pub struct Chess {
    pub board: [u8;64], //
    pub white_turn: bool,
    pub castle: u8,
    pub en_passant: u8, // position of jumped square
    pub half_move: u16,
    pub turn: u16,
    // lost piece | square | castle | en passant | half move clock
    pub irreversable_state: Vec<(u8, u8, u8, u8, u16)>

    
}

impl Chess {
    pub fn new() -> Chess {
        Chess::build(START_POSITION)
    }
    
    pub fn make_move(&mut self, movement: &Move) {
        let colour = self.colour_to_move();
        let mut eat_piece = self.board[movement.target_square as usize];
        let mut eat_square = movement.target_square;
        let (castle, en_passant, half_move) = (self.castle, self.en_passant, self.half_move);
        // irreversable stuff
        self.half_move += 1;
        self.en_passant = u8::MAX;
        if self.castle != castle::CASTLE_NONE {
            if movement.start_square == 0 || movement.target_square == 0 {self.castle &= !castle::CASTLE_W_Q}
            if movement.start_square == 7 || movement.target_square == 7 {self.castle &= !castle::CASTLE_W_K}
            if movement.start_square == 4 {self.castle &= !(castle::CASTLE_W_Q | castle::CASTLE_W_K)}
            if movement.start_square == 56 || movement.target_square == 56 {self.castle &= !castle::CASTLE_B_Q}
            if movement.start_square == 63 || movement.target_square == 63 {self.castle &= !castle::CASTLE_B_K}
            if movement.start_square == 60 {self.castle &= !(castle::CASTLE_B_Q | castle::CASTLE_B_K)}
        }
        if self.board[movement.target_square as usize] != NONE {
            self.half_move = 0;
            if movement.promotion != NONE {
                self.board[movement.start_square as usize] = movement.promotion | colour;
            }
        }
        else if self.board[movement.start_square as usize] == PAWN | colour {
            self.half_move = 0;
            let dist = movement.target_square as i8 - movement.start_square as i8;
            if dist == 16 || dist == -16 {
                self.en_passant = (movement.start_square as i8 + dist / 2) as u8;
            }
            else if movement.target_square == en_passant {
                // eats en passant
                eat_square = movement.start_square / 8 * 8 + movement.target_square % 8;
                eat_piece = self.board[eat_square as usize];
                self.board[eat_square as usize] = NONE;
            } else if movement.promotion != NONE {
                self.board[movement.start_square as usize] = movement.promotion | colour;
            }
        }
        else if self.board[movement.start_square as usize] == KING | colour {
            let dist = movement.target_square as i8 - movement.start_square as i8;
            if dist == 2 {
                self.board[movement.start_square as usize + 1] = ROCK | colour;
                self.board[movement.start_square as usize + 3] = NONE;
                eat_square = movement.start_square + 3;
                eat_piece = ROCK | colour;
            } else if dist == -2 {
                self.board[movement.start_square as usize - 1] = ROCK | colour;
                self.board[movement.start_square as usize - 4] = NONE;
                eat_square = movement.start_square - 4;
                eat_piece = ROCK | colour;
            }
        }
        // reversable stuff
        self.irreversable_state.push((eat_piece, eat_square, castle, en_passant, half_move));
        self.board[movement.target_square as usize] = self.board[movement.start_square as usize];
        self.board[movement.start_square as usize] = NONE;
        self.white_turn = !self.white_turn;
        self.turn += self.white_turn as u16; // inc if black end turn
    }

    pub fn unmake_move(&mut self, movement: &Move) {
        self.turn -= self.white_turn as u16;
        self.white_turn = !self.white_turn;
        let colour = self.colour_to_move();

        let (eat_piece, eat_square, castle, en_passant, half_move) = self.irreversable_state.pop().unwrap();
        self.castle = castle;
        self.en_passant = en_passant;
        self.half_move = half_move;
        self.board[movement.start_square as usize] = self.board[movement.target_square as usize];
        self.board[movement.target_square as usize] = NONE;
        self.board[eat_square as usize] = eat_piece;
        if self.board[movement.start_square as usize] == KING | colour {
            let dist = movement.target_square as i8 - movement.start_square as i8;
            if dist == 2 {
                self.board[movement.start_square as usize + 1] = NONE;
            } else if dist == -2 {
                self.board[movement.start_square as usize - 1] = NONE;
            }
        }
        if movement.promotion != NONE {
            self.board[movement.start_square as usize] = PAWN | colour;
        }
    }

    pub fn colour_to_move(&self) -> u8 {
        if self.white_turn {WHITE} else {BLACK}
    }
    pub fn colour_opponent(&self) -> u8 {
        if !self.white_turn {WHITE} else {BLACK}
    }
} 

pub mod utils {
    pub fn number_from_0_9(number: char) -> usize {
        number.to_digit(10).unwrap() as usize
    }
    pub fn number_from_a_h(letter: char) -> usize {
        letter as usize - 'a' as usize
    }
    pub fn square_from_text(letter: char, number: char) -> u8 {
        ((number_from_0_9(number)-1) * 8 + number_from_a_h(letter)) as u8
    }
}