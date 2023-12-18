#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Piece(u8);
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct  PieceType(u8);
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct  Colour(u8);
pub const NONE: Piece= Piece(0);
pub const NONE_COLOUR: Colour= Colour(0);
pub const NONE_TYPE: PieceType= PieceType(0);

pub const KING: PieceType= PieceType(1);
pub const KNIGHT: PieceType= PieceType(2);
pub const PAWN: PieceType= PieceType(3);
pub const BISHOP: PieceType= PieceType(4);
pub const ROOK: PieceType= PieceType(5);
pub const QUEEN: PieceType= PieceType(6);

pub const WHITE: Colour= Colour(8);
pub const BLACK: Colour= Colour(16);
impl Piece {
    pub fn new(piece: PieceType, colour: Colour) -> Piece {Piece(piece.0 | colour.0)}
    pub fn get_type(self) -> PieceType {PieceType(self.0 & !(WHITE.0 | BLACK.0))}
    pub fn get_colour(self) -> Colour {Colour(self.0 & (WHITE.0 | BLACK.0))}
    pub fn is_type(self, piece: PieceType) -> bool {self.get_type().0 == piece.0}
    pub fn is_colour(self, colour: Colour) -> bool {self.0 & colour.0 != 0}
    
    pub fn from_symbol(symbol: char) -> Piece {
        let piece = match symbol.to_ascii_lowercase() {
            'p' => PAWN, 'n' => KNIGHT, 'b' => BISHOP,
            'r' => ROOK, 'q' => QUEEN, 'k' => KING, _ => unreachable!(),
        };
        Piece::new(piece, if symbol.is_ascii_uppercase() {WHITE} else {BLACK})    
    }
    pub fn symbol(self: Piece) -> char {
        let symbol = match self.get_type() {
            PAWN => 'p', KNIGHT => 'n', BISHOP => 'b',
            ROOK => 'r', QUEEN => 'q', KING => 'k', _ => unreachable!(),
        };
        if self.is_colour(WHITE) {symbol.to_ascii_uppercase()} else {symbol}
    }
}
impl PieceType {
    pub fn is_sliding(self) -> bool {self.0 >= 4}
    pub fn get_sliding_indices(self) -> std::ops::Range<usize> {
        match self {
            BISHOP => 4..8,
            ROOK => 0..4,
            QUEEN => 0..8,
            _ => unreachable!(),
        }
    }
    pub fn piece_index(self) -> usize {return (self.0 - 1).into()}
}
impl Colour {
    pub fn new(is_white: bool) -> Colour {if is_white {WHITE} else {BLACK}}
    pub fn opponent(self) -> Colour {Colour(self.0 ^ (WHITE.0+BLACK.0))}
    pub fn colour_index(self) -> usize {match self {   
        WHITE => 0,
        BLACK => 1,
        _ => unreachable!(),
    }}
}