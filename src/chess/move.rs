
use super::*;

// FFFF TTTTTT SSSSSS
// F = flag, T = target, S = start
pub const NO_FLAG: u8 = 0b0000;
pub const EN_PASSANT_FLAG: u8 = 0b0001;
pub const CASTLE_FLAG: u8 = 0b0010;
pub const DOUBLE_PUSH_FLAG: u8 = 0b0011;
pub const PROMOTE_TO_QUEEN_FLAG: u8 = 0b0100;
pub const PROMOTE_TO_KNIGHT_FLAG: u8 = 0b0101;
pub const PROMOTE_TO_ROOK_FLAG: u8 = 0b0110;
pub const PROMOTE_TO_BISHOP_FLAG: u8 = 0b0111;

pub const NONE_MOVE: Move = Move { value: 0 };

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Move {
    value: u16,
}

impl Move {
    pub fn new(start: Square, target: Square, flag: u8) -> Move {
        Move { value: start as u16 | (target as u16) << 6 | (flag as u16) << 12 }
    }

    pub fn start(self) -> Square {(self.value & 0b0000000000111111) as Square}
    pub fn target(self) -> Square {((self.value & 0b0000111111000000) >> 6) as Square}
    pub fn flag(self) -> u8 {(self.value >> 12) as u8}

    pub fn promotion_type(self) -> PieceType {
        match self.flag() {
            PROMOTE_TO_ROOK_FLAG => ROOK,
            PROMOTE_TO_QUEEN_FLAG => QUEEN,
            PROMOTE_TO_BISHOP_FLAG => BISHOP,
            PROMOTE_TO_KNIGHT_FLAG => KNIGHT,
            _ => unreachable!()
        }
    }
    pub fn promotion_flag(piece_type: PieceType) -> u8 {
        match piece_type {
            QUEEN => PROMOTE_TO_QUEEN_FLAG,
            ROOK => PROMOTE_TO_ROOK_FLAG,
            BISHOP => PROMOTE_TO_BISHOP_FLAG,
            KNIGHT => PROMOTE_TO_KNIGHT_FLAG,
            _ => unreachable!()
        }
    }
    pub fn is_promotion(self) -> bool {self.value & 0b0100000000000000 != 0}
    pub fn is_king_castling(self) -> bool {self.target() % 8 == 6}

    pub fn from_text(chess: &Chess, text: &str) -> Move {
        let mut chars = text.chars();
        
        let start = square_from_text(chars.next().unwrap(), chars.next().unwrap());
        let target = square_from_text(chars.next().unwrap(), chars.next().unwrap());
        let promotion = chars.next().map(|symbol| PieceType::from_symbol(symbol));
        
        if let Some(promotion_type) = promotion {
            return Move::new(start, target, Move::promotion_flag(promotion_type));
        }
        else if chess.board[start as usize].is_type(KING) {
            if target as i8 - start as i8 == 2
            || target as i8 - start as i8 == -2
            {return Move::new(start, target, CASTLE_FLAG)}
        }
        else if chess.board[start as usize].is_type(PAWN) {
            if target as i8 - start as i8 == 16
            || target as i8 - start as i8 == -16
            {return Move::new(start, target, DOUBLE_PUSH_FLAG)}
        }
        
        return Move::new(start, target, NO_FLAG);
    }

    pub fn to_text(self) -> String {
        let (start, target) = (self.start(), self.target());
        match self.flag() {
            | NO_FLAG
            | CASTLE_FLAG
            | DOUBLE_PUSH_FLAG
            | EN_PASSANT_FLAG => format!("{}{}", square_to_text(start), square_to_text(target)),
            _ => format!("{}{}{}", square_to_text(start), square_to_text(target), self.promotion_type().symbol()),
        }
    }

}
