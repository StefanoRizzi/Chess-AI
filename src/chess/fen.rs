use super::*;

pub const START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

impl Chess {
    pub fn build(fen: &str) -> Chess {
        let mut iter = fen.trim().split(" ");
        
        let mut chess = Chess::new();
        // Piece Placement
        let fen_board = iter.next().expect("FEN board");
        let (mut rank, mut file) = (7, 0);
        for symbol in fen_board.chars() {
            if symbol == '/' {
                rank -= 1;
                file = 0;
            } else if symbol.is_ascii_digit() {
                file += symbol.to_digit(10).unwrap() as i8;
            } else {
                let square = rank * 8 + file;
                let piece = Piece::from_symbol(symbol);
                let colour = piece.get_colour();

                chess.new_piece(colour.colour_index(), piece.get_type(), square);
                chess.put_attack_and_update(piece.get_type(), colour, square);
                chess.board[square as usize] = piece;
                file += 1
            }
        }
        // Side to move
        chess.set_turn(iter.next() == Some("w"));
        // Castling ability
        let string = iter.next();
        if string != Some("-") {
            for ch in string.expect("castling ability").chars() {
                match ch {
                    'K' => chess.castling |= CASTLE_WHITE_KING,
                    'Q' => chess.castling |= CASTLE_WHITE_QUEEN,
                    'k' => chess.castling |= CASTLE_BLACK_KING,
                    'q' => chess.castling |= CASTLE_BLACK_QUEEN,
                    _ => panic!("castling ability")
                }
            }
        }
        // En passant target square
        let fen_en = iter.next().expect("FEN en passant");
        if fen_en != "-" {
            let mut chars = fen_en.chars();
            chess.en_passant = utils::square_from_text(chars.next().unwrap(), chars.next().unwrap());
        }
        // Halfmove clock
        if let Some(fen_half) = iter.next() {
            chess.half_move = fen_half.parse().unwrap();
        }
        // Fullmove counter
        if let Some(fen_full) = iter.next() {
            chess.full_turn = fen_full.parse().unwrap();
        }
        
        return chess
    }
}